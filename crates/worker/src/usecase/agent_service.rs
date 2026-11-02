//! 台本生成の LLM エージェント（[`agent_kit::agent`]）

pub(crate) use agent_kit::agent::*;

/// エージェントに渡す CMS の MCP ツール（読み取りだけ）。書き込み系（`RecordApi_update` など）は渡さない。
/// 保存は worker が LLM の出力を検証してから行う
pub(crate) const READ_ONLY_CMS_TOOLS: &[&str] = &[
    "CollectionApi_list",
    "CollectionApi_read",
    "RecordApi_list",
    "RecordApi_listWithRelations",
    "RecordApi_read",
    "RecordApi_referencedBy",
];
