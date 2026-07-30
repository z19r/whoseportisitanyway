/* global React */

function Footer() {
  return (
    <footer className="footer ws-wrap">
      <div className="footer-grid">
        <div className="footer-col">
          <div className="footer-heading">PROJECT</div>
          <a href="https://github.com/z19r/whoseportisitanyway" target="_blank" rel="noreferrer" data-umami-event="footer-link" data-umami-event-label="github">GitHub</a>
          <a href="https://github.com/z19r/whoseportisitanyway/releases" target="_blank" rel="noreferrer" data-umami-event="footer-link" data-umami-event-label="releases">Releases</a>
          <a href="https://crates.io/crates/whoseportisitanyway" target="_blank" rel="noreferrer" data-umami-event="footer-link" data-umami-event-label="crates-io">crates.io</a>
        </div>
        <div className="footer-col">
          <div className="footer-heading">DOCS</div>
          <a href="#features" data-umami-event="footer-link" data-umami-event-label="features">Features</a>
          <a href="#install" data-umami-event="footer-link" data-umami-event-label="install">Install</a>
          <a href="#faq" data-umami-event="footer-link" data-umami-event-label="faq">FAQ</a>
        </div>
        <div className="footer-col">
          <div className="footer-heading">LINKS</div>
          <a href="https://github.com/z19r" target="_blank" rel="noreferrer" data-umami-event="footer-link" data-umami-event-label="z19r">z19r</a>
          <a href="https://github.com/z19r/whoseportisitanyway/issues" target="_blank" rel="noreferrer" data-umami-event="footer-link" data-umami-event-label="issues">Issues</a>
        </div>
      </div>

      <div className="footer-meta">
        <span className="footer-copy">Made in Chicago, with <span className="heart">🫀</span> &copy;2026 z19r. All rights reserved.</span>
      </div>
    </footer>
  );
}

Object.assign(window, { Footer });
