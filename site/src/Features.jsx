/* global React */

const FEATURES = [
  {
    tag: 'CLASSIFY',
    title: 'Smart port classification',
    desc: 'Every listening port gets classified by service type, protocol, and purpose. Database, web server, dev tool, message broker — identified automatically.',
  },
  {
    tag: 'PROJECT',
    title: 'Project detection',
    desc: 'Traces each process back to its working directory and identifies the project by reading package.json, Cargo.toml, go.mod, and other manifest files.',
  },
  {
    tag: 'FRAMEWORK',
    title: 'Framework identification',
    desc: 'Inspects command lines, parent processes, and node_modules/.bin paths to detect Next.js, Vite, Django, Rails, and dozens more — even through npm script indirection.',
  },
  {
    tag: 'TUI',
    title: 'Interactive terminal UI',
    desc: 'Full ratatui-powered TUI with real-time refresh, filtering, sorting, and inline actions. Kill a process, copy a port, or jump to the project directory.',
  },
{
    tag: 'FAST',
    title: 'Zero startup cost',
    desc: 'Single Rust binary. No runtime, no daemon, no config file required. Scans /proc (Linux) or lsof (macOS) and renders in under 50ms.',
  },
];

function Features() {
  return (
    <section className="section ws-wrap" id="features">
      <div className="ws-sec-head">
        <div className="ws-sec-tag">01 · FEATURES</div>
        <h2>What it does.</h2>
      </div>

      <div className="feature-grid">
        {FEATURES.map((f) => (
          <div key={f.tag} className="feature-card">
            <div className="tag">{f.tag}</div>
            <h3>{f.title}</h3>
            <p>{f.desc}</p>
          </div>
        ))}
      </div>
    </section>
  );
}

Object.assign(window, { Features });
