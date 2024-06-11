import { useRef, useState } from "react";
import Index from "./CoeiroinkManager/Index.tsx";
import { useAtom } from "jotai";
import { navigatorLockedAtom } from "@/state.ts";
import Installing from "./CoeiroinkManager/Installing.tsx";
import Configure from "./CoeiroinkManager/Configure.tsx";

export type InstallContext = {
	edition: "cpu" | "gpu";
	version: string;
	path: string;
	desktopShortcut: boolean;
	startMenuShortcut: boolean;
};

const CoeiroinkManager: React.FC = () => {
	const [view, setView] = useState<"index" | "configure" | "installing">(
		"index",
	);
	const installContext = useRef<InstallContext>({
		edition: "cpu",
		version: "0.0.0",
		path: "",
		desktopShortcut: true,
		startMenuShortcut: true,
	});
	const [_navigatorLocked, setNavigatorLocked] = useAtom(navigatorLockedAtom);
	const configureCoeiroink = () => {
		setNavigatorLocked(true);
		setView("configure");
	};
	const installCoeiroink = (context: InstallContext) => {
		installContext.current = context;
		setView("installing");
	};
	if (view === "configure") {
		return (
			<Configure
				install={installCoeiroink}
				cancel={() => {
					setNavigatorLocked(false);
					setView("index");
				}}
			/>
		);
	}
	if (view === "installing") {
		return <Installing context={installContext.current} />;
	}
	return <Index configureCoeiroinkInstall={configureCoeiroink} />;
};

export default CoeiroinkManager;
