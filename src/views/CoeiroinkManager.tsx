import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const CoeiroinkManager: React.FC = () => {
	const [coeiroinkVersion, setCoeiroinkVersion] = useState<string | null>(null);
	useEffect(() => {
		invoke("get_coeiroink_version");
	}, []);
	return <div>こえいろ管理</div>;
};

export default CoeiroinkManager;
