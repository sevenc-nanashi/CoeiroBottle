import twNeumorphism from "tw-neumorphism";
import { Config } from "tailwindcss";

const config: Config = {
	content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
	theme: {
		extend: {
			colors: {
				coeiroink: "#f3f6fd",
			},
		},
	},
	plugins: [twNeumorphism],
};

export default config;
