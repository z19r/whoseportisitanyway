/* global React */

function Demo() {
  return (
    <section className="section ws-wrap" id="demo">
      <div className="ws-sec-head">
        <div className="ws-sec-tag">02 · DEMO</div>
        <h2>See it in action.</h2>
      </div>

      <div className="demo-terminal">
        <div className="demo-chrome">
          <div className="demo-dots"><span></span><span></span><span></span></div>
          <div className="demo-title">whoseportisitanyway · ~/projects</div>
          <div style={{ font: 'var(--t-mono)', fontSize: '11px', color: 'var(--c-lilac)' }}>[LIVE]</div>
        </div>
        <div className="demo-body">
<span className="header">{'  PORT   PID    PROCESS          PROJECT              FRAMEWORK'}</span>
<span className="header">{'  ─────  ─────  ───────────────  ───────────────────  ─────────────'}</span>
<span>{'  '}<span className="port">3000</span>{'   '}<span className="pid">14291</span>{'  node             '}<span className="project">my-saas-app</span>{'          '}<span className="framework">Next.js</span></span>
<span>{'  '}<span className="port">3001</span>{'   '}<span className="pid">14305</span>{'  node             '}<span className="project">my-saas-app</span>{'          '}<span className="framework">Next.js</span></span>
<span>{'  '}<span className="port">4000</span>{'   '}<span className="pid">8842</span>{'   beam.smp         '}<span className="project">phoenix-api</span>{'          '}<span className="framework">Phoenix</span></span>
<span>{'  '}<span className="port">5173</span>{'   '}<span className="pid">21003</span>{'  node             '}<span className="project">dashboard-ui</span>{'         '}<span className="framework">Vite</span></span>
<span>{'  '}<span className="port">5432</span>{'   '}<span className="pid">1204</span>{'   postgres         '}<span className="dim">—</span>{'                    '}<span className="dim">PostgreSQL</span></span>
<span>{'  '}<span className="port">6379</span>{'   '}<span className="pid">1198</span>{'   redis-server     '}<span className="dim">—</span>{'                    '}<span className="dim">Redis</span></span>
<span>{'  '}<span className="port">8000</span>{'   '}<span className="pid">9501</span>{'   python3          '}<span className="project">ml-service</span>{'           '}<span className="framework">Django</span></span>
<span>{'  '}<span className="port">8080</span>{'   '}<span className="pid">7220</span>{'   main             '}<span className="project">auth-proxy</span>{'           '}<span className="framework">Go</span></span>
<span>{'  '}<span className="port">9090</span>{'   '}<span className="pid">1502</span>{'   prometheus       '}<span className="dim">—</span>{'                    '}<span className="dim">Prometheus</span></span>
<span className="dim">{'\n  q quit · k kill · / filter · ? help · ↑↓ navigate'}</span>
        </div>
      </div>
    </section>
  );
}

Object.assign(window, { Demo });
