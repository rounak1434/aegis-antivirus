//! Download engine. A `Fetcher` abstracts transport so the pipeline can be
//! tested offline (`LocalFetcher`) and run over HTTPS in production
//! (`ReqwestFetcher`, with resume / timeout / retries / gzip).

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("http error: {0}")]
    Http(String),
    #[error("download exhausted {0} retries")]
    Retries(u32),
}

/// Fetches `url` into `dest`, returning the total byte count. Implementations
/// should resume a partial `dest` if possible.
pub trait Fetcher: Send + Sync {
    fn fetch(&self, url: &str, dest: &Path) -> Result<u64, DownloadError>;
}

/// Production HTTPS fetcher: byte-range resume, timeout, bounded retries, gzip.
pub struct ReqwestFetcher {
    client: reqwest::blocking::Client,
    retries: u32,
}

impl ReqwestFetcher {
    pub fn new(timeout: Duration, retries: u32) -> Result<Self, DownloadError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .gzip(true)
            .build()
            .map_err(|e| DownloadError::Http(e.to_string()))?;
        Ok(Self { client, retries })
    }
}

impl Fetcher for ReqwestFetcher {
    fn fetch(&self, url: &str, dest: &Path) -> Result<u64, DownloadError> {
        let mut last_err = DownloadError::Retries(self.retries);
        for _ in 0..=self.retries {
            match try_fetch(&self.client, url, dest) {
                Ok(n) => return Ok(n),
                Err(e) => last_err = e,
            }
        }
        Err(last_err)
    }
}

fn try_fetch(
    client: &reqwest::blocking::Client,
    url: &str,
    dest: &Path,
) -> Result<u64, DownloadError> {
    let resume_from = std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
    let mut req = client.get(url);
    if resume_from > 0 {
        req = req.header(reqwest::header::RANGE, format!("bytes={resume_from}-"));
    }
    let mut resp = req.send().map_err(|e| DownloadError::Http(e.to_string()))?;
    if !resp.status().is_success() && resp.status() != reqwest::StatusCode::PARTIAL_CONTENT {
        return Err(DownloadError::Http(format!("status {}", resp.status())));
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(resume_from > 0)
        .write(true)
        .open(dest)?;
    if resume_from > 0 {
        file.seek(SeekFrom::End(0))?;
    }
    let mut buf = [0u8; 64 * 1024];
    loop {
        let read = resp
            .read(&mut buf)
            .map_err(|e| DownloadError::Http(e.to_string()))?;
        if read == 0 {
            break;
        }
        file.write_all(&buf[..read])?;
    }
    Ok(file.metadata()?.len())
}

/// Test/offline fetcher: copies from a local base dir keyed by the URL's file
/// name. Supports resume by appending the remainder of the source.
pub struct LocalFetcher {
    base: PathBuf,
}

impl LocalFetcher {
    pub fn new(base: impl Into<PathBuf>) -> Self {
        Self { base: base.into() }
    }
}

impl Fetcher for LocalFetcher {
    fn fetch(&self, url: &str, dest: &Path) -> Result<u64, DownloadError> {
        let name = url.rsplit(['/', '\\']).next().unwrap_or(url);
        let src = self.base.join(name);
        let data = std::fs::read(&src)?;
        let resume_from = std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0) as usize;
        let mut file = OpenOptions::new()
            .create(true)
            .append(resume_from > 0)
            .write(true)
            .open(dest)?;
        if resume_from < data.len() {
            file.write_all(&data[resume_from..])?;
        }
        Ok(File::open(dest)?.metadata()?.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_fetch_copies() {
        let base = tempfile::tempdir().unwrap();
        std::fs::write(base.path().join("sig.bin"), b"payload-data").unwrap();
        let out = tempfile::tempdir().unwrap();
        let dest = out.path().join("sig.bin");
        let f = LocalFetcher::new(base.path());
        let n = f.fetch("https://feed/sig.bin", &dest).unwrap();
        assert_eq!(n, 12);
        assert_eq!(std::fs::read(&dest).unwrap(), b"payload-data");
    }

    #[test]
    fn local_fetch_resumes() {
        let base = tempfile::tempdir().unwrap();
        std::fs::write(base.path().join("r.bin"), b"0123456789").unwrap();
        let out = tempfile::tempdir().unwrap();
        let dest = out.path().join("r.bin");
        std::fs::write(&dest, b"01234").unwrap(); // partial
        let f = LocalFetcher::new(base.path());
        f.fetch("https://feed/r.bin", &dest).unwrap();
        assert_eq!(std::fs::read(&dest).unwrap(), b"0123456789");
    }
}
