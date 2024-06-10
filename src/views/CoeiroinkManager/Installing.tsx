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
			progress: number;
			total: number;
			current: string;
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

	const errorRef = useRef<string | null>(null);
	const [status, setStatus] = useState<"working" | "done" | "error">("working");
	const unlistenRef = useRef<(() => void) | null>(null);
	const [installProgress, setInstallProgress] = useState<InstallProgress>({
		type: "Initializing",
	});

	useEffect(() => {
		if (!invokedInstall.current) {
			setStatus("working");
			invokedInstall.current = true;

			listen<InstallProgress>("installing_coeiroink", (data) => {
				setInstallProgress(data.payload);
				if (data.payload.type === "Done") {
					setStatus("done");
				}
			}).then((unlisten) => {
				unlistenRef.current = unlisten;
			});

			invoke("install_coeiroink", { edition }).catch((e) => {
				errorRef.current = String(e);
				setStatus("error");
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
						`（${installProgress.progress} / ${installProgress.total}）`}
				</li>
				<li className={getClasses(3)}>
					インストール
					{installProgress.type === "Installing" && (
						<>
							（{installProgress.progress} / {installProgress.total || "？"}）
							<br />
							<span className="text-sm">{installProgress.current}</span>
						</>
					)}
				</li>
				<li className={getClasses(4)}>設定</li>
				<li className={getClasses(5)}>完了</li>
			</ul>

			{status === "working" && (
				<p>
					インストールには時間がかかります。お茶でも飲んでゆっくり待ちましょう。
				</p>
			)}
			{status === "error" && (
				<div className="pt-4">
					<p className="text-accent">
						エラーが発生しました：{errorRef.current}
					</p>
					<button
						type="button"
						onClick={() => window.location.reload()}
						className="button"
					>
						戻る
					</button>
				</div>
			)}
			{status === "done" && (
				<div className="pt-4">
					<p>インストールが完了しました。</p>
					<button
						type="button"
						onClick={() => window.location.reload()}
						className="button"
					>
						戻る
					</button>
				</div>
			)}
		</div>
	);
};

export default Installing;
