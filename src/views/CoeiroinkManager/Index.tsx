import { useContext, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Result } from "@oxi/result";
import semver from "semver";
import { Store } from "@/store.ts";
import * as Toast from "@radix-ui/react-toast";

type CoeiroinkVersion = {
  version: string;
  edition: "cpu" | "gpu";
};

const Index: React.FC<{
  configureCoeiroinkInstall: () => void;
}> = (props) => {
  const store = useContext(Store);

  const [coeiroinkVersion, setCoeiroinkVersion] = useState<Result<
    CoeiroinkVersion | null,
    string
  > | null>(null);
  useEffect(() => {
    const getCoeiroinkVersion = async () => {
      setCoeiroinkVersion(
        (
          await Result.from(() =>
            invoke<CoeiroinkVersion | null>("get_coeiroink_version")
          )
        ).mapErr(String)
      );
    };

    getCoeiroinkVersion();
  }, []);
  const [latestVersion, setLatestVersion] = useState<Result<
    {
      coeiroink: string | null;
      coeirobottle: string | null;
    },
    string
  > | null>(null);
  useEffect(() => {
    const fetchLatestVersion = async () => {
      setLatestVersion(
        (
          await Result.from(() =>
            invoke<{
              coeiroink: string | null;
              coeirobottle: string | null;
            }>("fetch_latest_version")
          )
        ).mapErr(String)
      );
    };

    fetchLatestVersion();
  }, []);

  const coeiroinkVersionStatus: "latest" | "updatable" | "none" = (() => {
    if (!coeiroinkVersion?.isOk()) {
      return "none";
    }
    if (!latestVersion?.isOk()) {
      return "none";
    }
    const coeiroinkVersionInner = coeiroinkVersion.unwrap();
    const latestVersionInner = latestVersion.unwrap();
    if (!(coeiroinkVersionInner && latestVersionInner.coeiroink)) {
      return "none";
    }
    if (
      semver.gt(latestVersionInner.coeiroink, coeiroinkVersionInner.version)
    ) {
      return "updatable";
    }
    return "latest";
  })();

  const [coeiroinkPath, setCoeiroinkPath] = useState<string>("");

  useEffect(() => {
    const getCoeiroinkPath = async () => {
      await store.load();
      const path = await store.get<string | null>("coeiroink_root");

      setCoeiroinkPath(path ?? "");
    };

    getCoeiroinkPath();
  }, [store]);

  const [isRootChangeToastOpen, setIsRootChangeToastOpen] = useState(false);

  if (!coeiroinkVersion || !latestVersion) {
    return <div className="grid place-items-center">読み込み中...</div>;
  }

  return (
    <div className="flex flex-col gap-4">
      <p>
        Coeiroinkのバージョン：
        {coeiroinkVersion
          ? coeiroinkVersion.match(
              (version) =>
                version ? (
                  <span>
                    {version.version}（
                    {version.edition === "cpu" ? "CPU" : "GPU"}）
                  </span>
                ) : (
                  <span>（未登録）</span>
                ),
              (err) => <span className="text-accent">（{err}）</span>
            )
          : "..."}
      </p>
      <section className="flex flex-col gap-2">
        <h1>
          Coeiroinkを
          {coeiroinkVersionStatus === "latest"
            ? "再インストールする"
            : coeiroinkVersionStatus === "updatable"
            ? "更新する"
            : "インストールする"}
        </h1>
        <p>
          {coeiroinkVersionStatus === "latest"
            ? "最新のCoeiroinkがインストールされています。"
            : coeiroinkVersionStatus === "updatable"
            ? "新しいバージョンのCoeiroinkが利用可能です。"
            : "Coeiroinkがインストールされていません。"}
        </p>

        <button
          type="button"
          className="button"
          onClick={props.configureCoeiroinkInstall}
        >
          {coeiroinkVersionStatus === "latest" ? "再" : ""}インストールする
        </button>
      </section>
      <section className="flex flex-col gap-2">
        <h1>
          Coeiroinkの場所を
          {coeiroinkVersionStatus === "none" ? "登録する" : "変更する"}
        </h1>
        {coeiroinkVersionStatus === "none" ? (
          <p>
            既にCoeiroinkがインストールされている場合は、その場所を登録することができます。{" "}
          </p>
        ) : (
          <p>
            CoeiroBottleが使うCoeiroinkの場所を変更することができます。基本的にはこの設定を変更する必要はありません。
            <br />
            <strong>注意：</strong>
            この設定を変更してもCoeiroink自体は移動しません。
          </p>
        )}
        <input
          type="text"
          value={coeiroinkPath}
          onChange={(e) => setCoeiroinkPath(e.target.value)}
          className="input"
        />

        <Toast.Provider>
          <button
            type="button"
            className="button"
            onClick={async () => {
              await store.set("coeiroink_root", coeiroinkPath);
              await store.save();
              setIsRootChangeToastOpen(true);
            }}
          >
            確定
          </button>

          <Toast.Root
            open={isRootChangeToastOpen}
            onOpenChange={setIsRootChangeToastOpen}
            className="toast-root"
            onClick={() => setIsRootChangeToastOpen(false)}
          >
            <Toast.Description>変更を保存しました。</Toast.Description>
          </Toast.Root>

          <Toast.Viewport />
        </Toast.Provider>
      </section>
    </div>
  );
};

export default Index;
