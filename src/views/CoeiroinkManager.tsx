import { useRef, useState } from "react";
import Index from "./CoeiroinkManager/Index.tsx";
import { useAtom } from "jotai";
import { navigatorLockedAtom } from "@/state.ts";
import Installing from "./CoeiroinkManager/Installing.tsx";

const CoeiroinkManager: React.FC = () => {
	const [view, setView] = useState<"index" | "installing" | "installed">(
		"index",
	);
	const installContext = useRef<{
		edition: "cpu" | "gpu";
		version: string;
	} | null>(null);
	const [_navigatorLocked, setNavigatorLocked] = useAtom(navigatorLockedAtom);
	const installCoeiroink = ({
		edition,
		version,
	}: { edition: "cpu" | "gpu"; version: string }) => {
		setNavigatorLocked(true);
		installContext.current = { edition, version };
		setView("installing");
	};
	return view === "installing" ? (
		installContext.current && (
			<Installing
				edition={installContext.current.edition}
				version={installContext.current.version}
			/>
		)
	) : (
		<Index installCoeiroink={installCoeiroink} />
	);
};

export default CoeiroinkManager;
