//! kafru のキューをまたいで trace context を引き継ぐ（[`agent_kit::trace_context`]）

pub(crate) use agent_kit::trace_context::*;

#[cfg(test)]
pub(crate) mod tests {
    pub(crate) use agent_kit::testing::test_subscriber;
}
