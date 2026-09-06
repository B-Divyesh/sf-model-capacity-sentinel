# Demo sandbox

Open `/demo` or use **Try it with sample data** on the landing page.

The sample contains two realistic model API probes: a healthy North America
chat endpoint and a Europe structured-output endpoint with a 429 capacity
failure. It also includes an open availability alert and a recovered latency
alert.

Demo changes use only the browser-local `demo:capacity-sentinel:summary`
storage namespace. Demo mode does not call `/api/*`, read a project access
code, or write to the runner's SQLite database. **Reset demo** restores the
shipped sample. **Start for real** leaves `/demo` and shows the separate
access-code screen.
