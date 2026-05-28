/* global React */

function Nav({ theme, onToggleTheme }) {
  const sections = [
    { id: 'features', label: '01 · FEATURES' },
    { id: 'demo',     label: '02 · DEMO' },
    { id: 'install',  label: '03 · INSTALL' },
    { id: 'faq',      label: '04 · FAQ' },
  ];

  const isLight = theme === 'light';

  return (
    <nav className="wp-nav">
      <a href="#top" className="wp-nav-brand">
        <span className="dot">&gt;</span> whoseportisitanyway
      </a>

      <div className="wp-nav-links">
        {sections.map((s) => (
          <a key={s.id} href={'#' + s.id}>{s.label}</a>
        ))}
      </div>

      <div className="wp-nav-right">
        <span className="ws-badge ws-badge--acid"><span className="pulse"></span>v1.0.0</span>
        <button className="theme-toggle" onClick={onToggleTheme} aria-label="Toggle theme">
          <span className="dot"></span>
          {isLight ? 'LIGHT' : 'NIGHT'}
        </button>
        <a className="ws-btn ws-btn--sm ws-btn--ghost" href="https://github.com/z19r/whoseportisitanyway" target="_blank" rel="noreferrer">GITHUB</a>
      </div>
    </nav>
  );
}

Object.assign(window, { Nav });
