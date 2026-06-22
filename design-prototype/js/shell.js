/* Aegis shared app chrome — injects window bar, sidebar nav, topbar.
   Usage: data attributes on <body data-shell data-active="dashboard"
   data-title="..." data-crumb="..."> then <script src="js/shell.js"></script> */
(function () {
  var I = {
    dashboard: '<path d="M3 13h8V3H3zM13 21h8V8h-8zM13 3v3h8V3zM3 21h8v-5H3z"/>',
    scan:      '<circle cx="11" cy="11" r="7"/><path d="M21 21l-4.3-4.3"/>',
    threats:   '<path d="M12 2l8 4v6c0 5-3.5 8-8 10-4.5-2-8-5-8-10V6z"/><path d="M12 9v4M12 16h.01"/>',
    quarantine:'<rect x="3" y="4" width="18" height="4" rx="1"/><path d="M5 8v11a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V8"/><path d="M10 12h4"/>',
    realtime:  '<path d="M12 2l8 4v6c0 5-3.5 8-8 10-4.5-2-8-5-8-10V6z"/><path d="M9 12l2 2 4-4"/>',
    history:   '<path d="M3 3v5h5"/><path d="M3.05 13A9 9 0 1 0 6 5.3L3 8"/><path d="M12 7v5l3 2"/>',
    settings:  '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.6 1.6 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.6 1.6 0 0 0-2.7 1.1V21a2 2 0 0 1-4 0v-.1a1.6 1.6 0 0 0-2.7-1.1l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1A1.6 1.6 0 0 0 4 15a1.6 1.6 0 0 0-1.5-1H2a2 2 0 0 1 0-4h.1A1.6 1.6 0 0 0 4 8.6a1.6 1.6 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1A1.6 1.6 0 0 0 9 4V2a2 2 0 0 1 4 0v.1a1.6 1.6 0 0 0 2.7 1.1l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.6 1.6 0 0 0 1.1 2.7H22a2 2 0 0 1 0 4h-.1a1.6 1.6 0 0 0-1.5 1z"/>',
    arch:      '<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/>',
    widget:    '<rect x="4" y="3" width="16" height="18" rx="2"/><path d="M9 8h6"/>',
    shield:    '<path d="M12 2l8 4v6c0 5-3.5 8-8 10-4.5-2-8-5-8-10V6z"/>',
    home:      '<path d="M3 11l9-8 9 8"/><path d="M5 9v11h14V9"/>'
  };
  function ic(name) {
    return '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" ' +
      'stroke-linecap="round" stroke-linejoin="round">' + (I[name]||'') + '</svg>';
  }
  var NAV = [
    {g:'Overview', items:[
      {k:'home', href:'index.html', label:'Launcher', icon:'home'},
      {k:'dashboard', href:'dashboard.html', label:'Dashboard', icon:'dashboard'},
      {k:'arch', href:'architecture.html', label:'Architecture', icon:'arch'}
    ]},
    {g:'Protect', items:[
      {k:'scan', href:'scan.html', label:'Scan', icon:'scan'},
      {k:'threats', href:'threats.html', label:'Threats', icon:'threats', badge:'3'},
      {k:'quarantine', href:'quarantine.html', label:'Quarantine', icon:'quarantine'},
      {k:'realtime', href:'realtime.html', label:'Real-time', icon:'realtime'}
    ]},
    {g:'System', items:[
      {k:'widget', href:'widget.html', label:'Mini widget', icon:'widget'},
      {k:'settings', href:'settings.html', label:'Settings', icon:'settings'}
    ]}
  ];

  function buildSidebar(active) {
    var html = '<div class="brand"><div class="logo">'+ic('shield')+'</div>' +
      '<div><div class="bn">Aegis</div><div class="bv">v0.4.0 · open source</div></div></div>';
    NAV.forEach(function(sec){
      html += '<div class="nav-label">'+sec.g+'</div>';
      sec.items.forEach(function(it){
        html += '<a class="nav-item'+(it.k===active?' active':'')+'" href="'+it.href+'">'+
          ic(it.icon)+'<span>'+it.label+'</span>'+
          (it.badge?'<span class="badge num">'+it.badge+'</span>':'')+'</a>';
      });
    });
    html += '<div class="spacer"></div>';
    html += '<div class="shield-mini">'+ic('shield')+
      '<div><div class="sm-t">Protected</div><div class="sm-s">Real-time on · defs 2h ago</div></div></div>';
    return '<aside class="sidebar">'+html+'</aside>';
  }

  function buildWinbar() {
    return '<div class="winbar"><div class="wb-title"><span class="wb-dot"></span>'+
      'Aegis Security — running with elevated privileges</div>' +
      '<div class="win-ctrls">'+
      '<button title="Minimize"><svg width="11" height="11" viewBox="0 0 11 11" stroke="currentColor" stroke-width="1.3"><line x1="1" y1="6" x2="10" y2="6"/></svg></button>'+
      '<button title="Maximize"><svg width="11" height="11" viewBox="0 0 11 11" stroke="currentColor" stroke-width="1.3" fill="none"><rect x="1.5" y="1.5" width="8" height="8"/></svg></button>'+
      '<button class="close" title="Close"><svg width="11" height="11" viewBox="0 0 11 11" stroke="currentColor" stroke-width="1.3"><line x1="1.5" y1="1.5" x2="9.5" y2="9.5"/><line x1="9.5" y1="1.5" x2="1.5" y2="9.5"/></svg></button>'+
      '</div></div>';
  }

  function buildTopbar(title, crumb) {
    return '<header class="topbar">'+
      (crumb?'<span class="crumb">'+crumb+'</span>':'')+
      '<h1>'+title+'</h1>'+
      '<div class="tb-right">'+
        '<button class="icon-btn" title="Search (Ctrl K)"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><circle cx="11" cy="11" r="7"/><path d="M21 21l-4.3-4.3"/></svg></button>'+
        '<button class="icon-btn" title="Notifications"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><path d="M18 8a6 6 0 1 0-12 0c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.7 21a2 2 0 0 1-3.4 0"/></svg></button>'+
        '<span class="pill ok"><span class="dot"></span>Protected</span>'+
      '</div>'+
    '</header>';
  }

  document.addEventListener('DOMContentLoaded', function () {
    var body = document.body;
    if (!body.hasAttribute('data-shell')) return;
    var active = body.getAttribute('data-active') || '';
    var title  = body.getAttribute('data-title') || 'Aegis';
    var crumb  = body.getAttribute('data-crumb') || '';
    var inner  = body.innerHTML;

    body.innerHTML =
      buildWinbar() +
      '<div class="app has-winbar">' +
        buildSidebar(active) +
        '<div class="main">' + buildTopbar(title, crumb) +
          '<div class="content'+(body.getAttribute('data-wide')!==null && body.hasAttribute('data-wide')?' wide':'')+'" id="aegis-content">' +
            inner +
          '</div>' +
        '</div>' +
      '</div>';
  });

  window.AegisIcons = I;
  window.aegisIcon = ic;
})();
