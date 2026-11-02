//! 台本生成のエージェントに渡す client tool `fetch_url`（[`agent_kit::fetch_url_tool`]）

pub(crate) use agent_kit::fetch_url_tool::*;

#[cfg(test)]
pub(crate) mod tests {
    pub(crate) use agent_kit::testing::FakeFetcher;
}
