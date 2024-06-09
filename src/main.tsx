import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App.tsx";
import "./index.scss";
import "ui-neumorphism/dist/index.css";

ReactDOM.createRoot(document.getElementById("app")!).render(
	<React.StrictMode>
		<App />
	</React.StrictMode>,
);
