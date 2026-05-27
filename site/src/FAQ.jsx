/* global React */

const FAQS = [
  {
    q: 'What platforms are supported?',
    a: 'Linux (x86_64 and ARM64) and macOS (Intel and Apple Silicon). Linux uses /proc for fast scanning; macOS uses lsof.',
  },
  {
    q: 'How does project detection work?',
    a: 'Each listening process is traced back to its working directory. The tool then walks up the directory tree looking for manifest files — package.json, Cargo.toml, go.mod, pyproject.toml, mix.exs, and others — to identify the project name.',
  },
  {
    q: 'How does framework detection work?',
    a: 'Command lines, parent processes, and node_modules/.bin paths are inspected to detect frameworks like Next.js, Vite, Django, Rails, Phoenix, and dozens more — even through npm script indirection.',
  },
  {
    q: 'Can I kill a process from the TUI?',
    a: 'Yes. Select a row and press k to send SIGTERM. The TUI confirms before killing and refreshes the port list automatically.',
  },
{
    q: 'Is there a JSON / CLI output mode?',
    a: 'Yes. Run with --json to get structured output for scripting, or use --once for a single non-interactive snapshot.',
  },
];

function FAQ() {
  const [open, setOpen] = React.useState(null);

  return (
    <section className="section ws-wrap" id="faq">
      <div className="ws-sec-head">
        <div className="ws-sec-tag">04 · FAQ</div>
        <h2>Questions.</h2>
      </div>

      <div className="faq-list">
        {FAQS.map((f, i) => (
          <div
            key={i}
            className={'faq-item' + (open === i ? ' is-open' : '')}
            onClick={() => setOpen(open === i ? null : i)}
          >
            <div className="faq-q">
              <span>{f.q}</span>
              <span className="faq-toggle">{open === i ? '−' : '+'}</span>
            </div>
            {open === i && <div className="faq-a">{f.a}</div>}
          </div>
        ))}
      </div>
    </section>
  );
}

Object.assign(window, { FAQ });
