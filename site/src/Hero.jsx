/* global React */

function CopyChip({ cmd }) {
  const [copied, setCopied] = React.useState(false);
  const onCopy = () => {
    navigator.clipboard?.writeText(cmd).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1400);
    });
  };
  return (
    <div className="copy-chip">
      <span className="prompt">$</span>
      <span className="cmd">{cmd}</span>
      <button className={'copy-btn' + (copied ? ' is-copied' : '')} onClick={onCopy}>
        {copied ? 'COPIED' : 'COPY'}
      </button>
    </div>
  );
}

function Hero() {
  return (
    <section className="hero ws-wrap" id="top">
      <div className="eyebrow-row">
        <span className="bar"></span>
        <span>// WHOSEPORTISITANYWAY · v1.0.1 · RUST</span>
      </div>

      <h1 className="display">
        Know your<br />
        <span className="acc">ports</span>.
      </h1>

      <p className="sub">
        A cross-platform terminal UI that discovers which ports are in use,
        identifies the owning process, detects the <b>project</b> and <b>framework</b>,
        and lets you kill or inspect any listener with a keystroke.
      </p>

      <div className="hero-cta">
        <CopyChip cmd="cargo install whoseportisitanyway" />
        <a className="ws-btn ws-btn--primary" href="#install">INSTALL</a>
        <a className="ws-btn ws-btn--ghost" href="https://github.com/z19r/whoseportisitanyway" target="_blank" rel="noreferrer">VIEW ON GITHUB</a>
      </div>

      <div className="hero-badges">
        <span className="ws-badge ws-badge--acid"><span className="pulse"></span>BUILD · PASSING</span>
        <span className="ws-badge ws-badge--royal">RUST</span>
        <span className="ws-badge ws-badge--mag">MIT</span>
        <span className="ws-badge ws-badge--bone">LINUX + MACOS</span>
      </div>
    </section>
  );
}

Object.assign(window, { Hero, CopyChip });
