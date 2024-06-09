import { useEffect, useState } from "react";
import { version } from "../package.json" assert { type: "json" };
import CoeiroinkManager from "./views/CoeiroinkManager";
import clsx from "clsx";
import { Result } from "@oxi/result";
import { invoke } from "@tauri-apps/api/core";
import * as Tooltip from "@radix-ui/react-tooltip";
import { useAtom } from "jotai";
import { navigatorLockedAtom } from "./state";

const App: React.FC = () => {
	const [view, setView] = useState("coeiroinkManager");
	const [navigatorLocked] = useAtom(navigatorLockedAtom);

	const [isCoeiroinkInstalled, setIsCoeiroinkInstalled] =
		useState<boolean>(false);
	useEffect(() => {
		const checkIfInstalled = async () => {
			setIsCoeiroinkInstalled(
				(
					await Result.from(() =>
						invoke<string | null>("get_coeiroink_version"),
					)
				).mapOr(false, (v) => !!v),
			);
		};

		checkIfInstalled();
	}, []);

	return (
		<div className="overflow-hidden w-screen h-screen grid grid-rows-[auto_1fr_auto]">
			<header className="grid grid-cols-2 gap-2 m-2">
				<button
					type="button"
					className={clsx("button", view === "coeiroinkManager" && "active")}
					onClick={() => setView("coeiroinkManager")}
					disabled={navigatorLocked}
				>
					Coeiroinkを管理する
				</button>

				<Tooltip.Provider delayDuration={0} skipDelayDuration={0}>
					<Tooltip.Root>
						<Tooltip.Trigger asChild>
							<button
								type="button"
								className={clsx("button", view === "myCoe" && "active")}
								onClick={() => setView("myCoe")}
								disabled={!isCoeiroinkInstalled || navigatorLocked}
							>
								MyCoeを管理する
							</button>
						</Tooltip.Trigger>
						{navigatorLocked || (
							<Tooltip.Content
								sideOffset={5}
								side="top"
								align="center"
								className="tooltip-content"
							>
								Coeiroinkがインストールされていません
							</Tooltip.Content>
						)}
					</Tooltip.Root>
				</Tooltip.Provider>
			</header>

			<main className="p-2 m-2 mb-0 rounded-md">
				{view === "coeiroinkManager" ? (
					<CoeiroinkManager />
				) : (
					<div>MyCoe管理</div>
				)}
			</main>
			<footer className="grid place-items-center opacity-50 py-2 text-xs">
				<p>
					CoeiroBottle - Coeiroink Helper v{version} - &copy; 2024{" "}
					<a
						href="https://sevenc7c.com"
						target="_blank"
						className="text-sevenc7c"
						rel="noreferrer"
					>
						Nanashi.
					</a>{" "}
					-{" "}
					<a
						href="https://github.com/sevenc-nanashi/CoeiroBottle"
						target="_blank"
						rel="noreferrer"
					>
						sevenc-nanashi/CoeiroBottle
					</a>
				</p>
			</footer>
		</div>
	);
};

export default App;
