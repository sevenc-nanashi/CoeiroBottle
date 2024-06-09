import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import clsx from "clsx";
import { useEffect, useRef, useState } from "react";

type InstallProgress =
	| {
			type: "Initializing";
	  }
	| {
			type: "Downloading";
			progress: number;
			total: number;
	  }
	| {
			type: "Extracting";
			progress: number;
			total: number;
	  }
	| {
			type: "Installing";
	  }
	| {
			type: "Configuring";
	  }
	| {
			type: "Done";
	  };

const typeToLevel = (type: InstallProgress["type"]) =>
	[
		"Initializing",
		"Downloading",
		"Extracting",
		"Installing",
		"Configuring",
		"Done",
	].indexOf(type);

const toMib = (bytes: number) => (bytes / 1024 / 1024).toFixed(2);

const Installing: React.FC<{ edition: "cpu" | "gpu"; version: string }> = ({
	edition,
	version,
}) => {
	const invokedInstall = useRef(false);

	const unlistenRef = useRef<(() => void) | null>(null);
	const [installProgress, setInstallProgress] = useState<InstallProgress>({
		type: "Initializing",
	});

	useEffect(() => {
		if (!invokedInstall.current) {
			invokedInstall.current = true;

			listen<InstallProgress>("installing_coeiroink", (data) => {
				setInstallProgress(data.payload);
			}).then((unlisten) => {
				unlistenRef.current = unlisten;
			});

			invoke("install_coeiroink", { edition }).catch((e) => {
				console.error(e);
			});
		}

		return () => {
			unlistenRef.current?.();
		};
	}, [edition]);

	const progressLevel = typeToLevel(installProgress.type);

	const getClasses = (level: number) =>
		clsx(
			"list-disc ml-8",
			level < progressLevel && "opacity-50",
			level === progressLevel && "text-accent",
		);

	return (
		<div className="flex flex-col gap-4">
			<h1>インストール中...</h1>
			<p>
				バージョン：v{version}（{edition === "cpu" ? "CPU" : "GPU"}）
			</p>
			<ul>
				<li className={getClasses(0)}>初期化</li>
				<li className={getClasses(1)}>
					ダウンロード
					{installProgress.type === "Downloading" &&
						`（${toMib(installProgress.progress)}MiB / ${toMib(
							installProgress.total,
						)}MiB）`}
				</li>
				<li className={getClasses(2)}>
					解凍
					{installProgress.type === "Extracting" &&
						`（${installProgress.progress} / ${installProgress.total}MiB）`}
				</li>
				<li className={getClasses(3)}>インストール</li>
				<li className={getClasses(4)}>設定</li>
				<li className={getClasses(5)}>完了</li>
			</ul>
		</div>
	);
};

export default Installing;
