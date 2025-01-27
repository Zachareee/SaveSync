import { useEffect, useState } from "react";
import { invoke } from "./utils.ts";
import "./App.css";
import { Info } from "./types.ts";
function App() {
  const [services, setServices] = useState<Info[]>([]);
  useEffect(() => {
    invoke("get_plugins").then(setServices);
  }, []);
  return (
    <main className="container">
      <h1>Welcome to Tauri + React + Lua</h1>
      {services.map(({ name, description, author }, idx) => (
        <div key={idx}>
          <h2>{name}</h2>
          <p>Description: {description}</p>
          <span>Written by: {author}</span>
        </div>
      ))}
    </main>
  );
}
export default App;
