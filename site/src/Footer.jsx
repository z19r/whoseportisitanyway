/* global React */

function Footer() {
  return (
    <footer className="footer ws-wrap">
      <div className="footer-grid">
        <div className="footer-col">
          <div className="footer-heading">PROJECT</div>
          <a href="https://github.com/z19r/whoseportisitanyway" target="_blank" rel="noreferrer">GitHub</a>
          <a href="https://github.com/z19r/whoseportisitanyway/releases" target="_blank" rel="noreferrer">Releases</a>
          <a href="https://crates.io/crates/whoseportisitanyway" target="_blank" rel="noreferrer">crates.io</a>
        </div>
        <div className="footer-col">
          <div className="footer-heading">DOCS</div>
          <a href="#features">Features</a>
          <a href="#install">Install</a>
          <a href="#faq">FAQ</a>
        </div>
        <div className="footer-col">
          <div className="footer-heading">LINKS</div>
          <a href="https://github.com/z19r" target="_blank" rel="noreferrer">z19r</a>
          <a href="https://github.com/z19r/whoseportisitanyway/issues" target="_blank" rel="noreferrer">Issues</a>
        </div>
      </div>

      <div className="footer-meta">
        <span>MADE IN CHICAGO, WITH 🫀 &copy; {new Date().getFullYear()} z19r. All rights reserved</span>
      </div>
    </footer>
  );
}

Object.assign(window, { Footer });
