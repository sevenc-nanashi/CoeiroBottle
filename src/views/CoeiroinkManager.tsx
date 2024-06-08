import React, { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

const CoeiroinkManager: React.FC = () => {
  useEffect(() => {
    invoke("check_version");
  }, []);
  return <div>こえいろ管理</div>;
};

export default CoeiroinkManager;
