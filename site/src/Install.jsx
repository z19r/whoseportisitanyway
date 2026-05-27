/* global React, CopyChip */

function Install() {
  return (
    <section className="section ws-wrap" id="install">
      <div className="ws-sec-head">
        <div className="ws-sec-tag">03 · INSTALL</div>
        <h2>Get started.</h2>
      </div>

      <div style={{ marginBottom: 'var(--s-7)' }}>
        <CopyChip cmd="cargo install whoseportisitanyway" />
      </div>

      <div className="install-targets">
        <a className="install-target" href="https://github.com/z19r/whoseportisitanyway/releases/latest" style={{ textDecoration: 'none', border: 'var(--b-med) solid var(--fg)' }}>
          <div className="icon">&#x1F427;</div>
          <div className="info">
            <div className="name">Linux x86_64</div>
            <div className="arch">x86_64-unknown-linux-gnu</div>
          </div>
        </a>
        <a className="install-target" href="https://github.com/z19r/whoseportisitanyway/releases/latest" style={{ textDecoration: 'none', border: 'var(--b-med) solid var(--fg)' }}>
          <div className="icon">&#x1F427;</div>
          <div className="info">
            <div className="name">Linux ARM64</div>
            <div className="arch">aarch64-unknown-linux-gnu</div>
          </div>
        </a>
        <a className="install-target" href="https://github.com/z19r/whoseportisitanyway/releases/latest" style={{ textDecoration: 'none', border: 'var(--b-med) solid var(--fg)' }}>
          <div className="icon">&#x1F34E;</div>
          <div className="info">
            <div className="name">macOS Intel</div>
            <div className="arch">x86_64-apple-darwin</div>
          </div>
        </a>
        <a className="install-target" href="https://github.com/z19r/whoseportisitanyway/releases/latest" style={{ textDecoration: 'none', border: 'var(--b-med) solid var(--fg)' }}>
          <div className="icon">&#x1F34E;</div>
          <div className="info">
            <div className="name">macOS Apple Silicon</div>
            <div className="arch">aarch64-apple-darwin</div>
          </div>
        </a>
      </div>
    </section>
  );
}

Object.assign(window, { Install });
