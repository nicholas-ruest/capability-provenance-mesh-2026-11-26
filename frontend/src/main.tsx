import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { ProductApp } from "./ProductApp";
import "./styles.css";

const element = document.getElementById("root");
if (!element) throw new Error("Application root is missing");
createRoot(element).render(<StrictMode><ProductApp /></StrictMode>);
