import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Info } from "./types.ts";
function App() {
  const [services, setServices] = useState<Info[]>([]);
  useEffect(() => {
    invoke<Info[]>("get_services").then(setServices);
  }, []);
  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>
      {services.map(({ name, description, author }) => (
        <div>
          <h2>{name}</h2>
          <p>{description}</p>
          <span>{author}</span>
        </div>
      ))}
    </main>
  );
}
export default App;
