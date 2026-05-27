/* global React, ReactDOM, Nav, Hero, Features, Demo, Install, FAQ, Footer */

function App() {
  const [theme, setTheme] = React.useState(() => {
    try {
      var saved = localStorage.getItem('wp-theme');
      if (saved === 'light' || saved === 'dark') return saved;
    } catch (_) {}
    return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches
      ? 'dark'
      : 'light';
  });

  const toggleTheme = React.useCallback(() => {
    setTheme((prev) => {
      var next = prev === 'dark' ? 'light' : 'dark';
      document.documentElement.setAttribute('data-theme', next);
      try { localStorage.setItem('wp-theme', next); } catch (_) {}
      return next;
    });
  }, []);

  return (
    <React.Fragment>
      <Nav theme={theme} onToggleTheme={toggleTheme} />
      <Hero />
      <Features />
      <Demo />
      <Install />
      <FAQ />
      <Footer />
    </React.Fragment>
  );
}

ReactDOM.createRoot(document.getElementById('root')).render(<App />);
