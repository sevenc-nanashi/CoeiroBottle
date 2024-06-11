import { useState, useEffect, useRef, useContext } from "react";
import type { InstallContext } from "../CoeiroinkManager.tsx";
import { invoke } from "@tauri-apps/api/core";
import * as Select from "@radix-ui/react-select";
import * as Checkbox from "@radix-ui/react-checkbox";
import { CheckIcon, ChevronDownIcon } from "@radix-ui/react-icons";
import path from "path-browserify";
import { Store } from "@/store.ts";

type DownloadInfo = {
	edition: "cpu" | "gpu";
	os: "windows" | "mac";
	version: string;
	link: string;
};

const Configure: React.FC<{
	install: (context: InstallContext) => void;
	cancel: () => void;
}> = (props) => {
	const store = useContext(Store);

	const [installPath, setInstallPath] = useState<string>("");
	const [coeiroinkVersions, setCoeiroinkVersions] = useState<DownloadInfo[]>(
		[],
	);
	const [coeiroinkToInstall, setCoeiroinkToInstall] = useState<string | null>(
		null,
	);
	const [desktopShortcut, setDesktopShortcut] = useState(true);
	const [startMenuShortcut, setStartMenuShortcut] = useState(true);

	const defaultInstallPathRoot = useRef("");

	useEffect(() => {
		const getCoeiroinkPath = async () => {
			await store.load();
			const currentRoot = await store.get<string | null>("coeiroink_root");

			if (currentRoot) {
				setInstallPath(currentRoot);
				defaultInstallPathRoot.current = path.dirname(currentRoot);
				return;
			}

			defaultInstallPathRoot.current = await invoke<string>(
				"default_install_path_root",
			);

			setInstallPath(`${defaultInstallPathRoot.current}/coeiroink-v2`);
		};

		getCoeiroinkPath();
	}, [store]);

	useEffect(() => {
		const getDownloadInfo = async () => {
			const info = await invoke<DownloadInfo[]>("fetch_coeiroink_versions");

			const windows = info.filter((v) => v.os === "windows");
			setCoeiroinkVersions(windows);
			const latest = windows[0];
			if (!latest) return;
			setCoeiroinkToInstall(`${latest.version}-cpu`);
		};

		getDownloadInfo();
	}, []);

	useEffect(() => {
		(async () => {
			if (!coeiroinkToInstall) return;

			const [version, edition] = coeiroinkToInstall.split("-");

			if (version === coeiroinkVersions[0].version) {
				const currentRoot = await store.get<string | null>("coeiroink_root");
				setDesktopShortcut(true);
				setStartMenuShortcut(true);

				if (currentRoot) {
					setInstallPath(currentRoot);
					return;
				}
				setInstallPath(`${defaultInstallPathRoot.current}/coeiroink-v2`);
				return;
			}
			setDesktopShortcut(false);
			setStartMenuShortcut(false);
			setInstallPath(
				`${defaultInstallPathRoot.current}/coeiroink-v${version}-${edition}`,
			);
		})();
	}, [coeiroinkToInstall, coeiroinkVersions, store]);

	const submit = (e: React.FormEvent) => {
		e.preventDefault();
		if (!coeiroinkToInstall) return;
		props.install({
			edition: coeiroinkToInstall.split("-")[1] as "cpu" | "gpu",
			version: coeiroinkToInstall.split("-")[0],
			path: installPath,
			desktopShortcut,
			startMenuShortcut,
		});
	};

	return (
		<form className="flex flex-col gap-4 h-full" onSubmit={submit}>
			<h1>インストール設定</h1>
			<section className="flex flex-col gap-2">
				<h2>バージョン</h2>
				<p>インストールするバージョンを選択します。</p>
				<Select.Root
					value={coeiroinkToInstall ?? undefined}
					onValueChange={setCoeiroinkToInstall}
					required
				>
					<Select.Trigger className="select-trigger">
						<Select.Value placeholder="..." />
						<Select.Icon className="select-icon">
							<ChevronDownIcon />
						</Select.Icon>
					</Select.Trigger>
					<Select.Portal>
						<Select.Content className="select-content">
							<Select.ScrollUpButton />
							<Select.Viewport className="select">
								{coeiroinkVersions.map(({ version, edition }) => (
									<Select.Item
										key={`${version}-${edition}`}
										value={`${version}-${edition}`}
										className="select-item"
									>
										<Select.ItemText>
											{version}（{edition.toUpperCase()}）
										</Select.ItemText>
									</Select.Item>
								))}
							</Select.Viewport>

							<Select.ScrollDownButton />
							<Select.Arrow />
						</Select.Content>
					</Select.Portal>
				</Select.Root>
			</section>
			<section className="flex flex-col gap-2">
				<h2>インストール先</h2>
				<p>インストール先を指定します。</p>
				<input
					type="text"
					className="input"
					value={installPath}
					onChange={(e) => setInstallPath(e.target.value)}
					onBlur={(e) => {
						if (
							e.target.value === "" ||
							!e.target.value.match(/^[a-zA-Z]:[\\/]/)
						) {
							setInstallPath(`${defaultInstallPathRoot.current}/coeiroink-v2`);
						}
					}}
				/>
			</section>
			<section className="flex flex-col gap-2">
				<h2>ショートカット</h2>

				<div className="flex">
					<Checkbox.Root
						className="checkbox-root"
						defaultChecked
						checked={desktopShortcut}
						onCheckedChange={(e) => setDesktopShortcut(e.valueOf() as boolean)}
						id="configure--desktop"
					>
						<Checkbox.Indicator className="checkbox-indicator">
							<CheckIcon />
						</Checkbox.Indicator>
					</Checkbox.Root>
					<label className="checkbox-label" htmlFor="configure--desktop">
						デスクトップにショートカットを作成する
					</label>
				</div>
				<div className="flex">
					<Checkbox.Root
						className="checkbox-root"
						defaultChecked
						checked={startMenuShortcut}
						onCheckedChange={(e) =>
							setStartMenuShortcut(e.valueOf() as boolean)
						}
						id="configure--start-menu"
					>
						<Checkbox.Indicator className="checkbox-indicator">
							<CheckIcon />
						</Checkbox.Indicator>
					</Checkbox.Root>
					<label className="checkbox-label" htmlFor="configure--start-menu">
						スタートメニューにショートカットを作成する
					</label>
				</div>
			</section>
			<div className="flex-grow" />
			<section className="grid gap-2 grid-cols-2">
				<button type="button" className="button" onClick={props.cancel}>
					キャンセル
				</button>

				<button type="submit" className="button" disabled={!coeiroinkToInstall}>
					インストール
				</button>
			</section>
		</form>
	);
};

export default Configure;
