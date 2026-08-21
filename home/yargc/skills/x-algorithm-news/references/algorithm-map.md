# public algorithm map

The repository analysis in `x-algorithm-news-account-analysis.md` is the
working Turkish explanation. Verify code claims against the pinned x-algorithm
checkout before changing heuristics.

| signal or gate | public implementation | use in XPatla |
|---|---|---|
| candidate retrieval and filtering | `home-mixer/candidate_pipeline/phoenix_candidate_pipeline.rs` | keep discovery, filtering and ranking separate |
| action score composition | `home-mixer/scorers/ranking_scorer.rs` | model opportunity surfaces; do not copy private predictions |
| configurable coefficients | `home-mixer/params/param.rs` | record the checked revision and defaults |
| video exclusion | `home-mixer/filters/video_filter.rs` | video is a candidate type, not an automatic winner |
| video quality view | `home-mixer/util/candidates_util.rs` | a duration-qualified predicted surface, not a guarantee |
| reply/repost OON filtering | `home-mixer/filters/oon_retweet_reply_filter.rs` | publish news as an original post |
| conversation deduplication | `home-mixer/filters/dedup_conversation_filter.rs` | prevent repeated event/ancestor candidates |
| semantic diversity | `vm-ranker/dpp.rs`, `vm-ranker/scoring/dpp_model.rs` | penalize self-competing repeated topics/media |
| visibility and safety | `visibility-filtering/rules/registry.rs` | keep sensitive and policy-uncertain media out of autopilot |

Current public defaults expose `photo_expand`, `video_open` and `vqv` as
prediction heads with small coefficients. These heads operate on predicted
probabilities/continuous values. They are not raw count multipliers and do not
prove that every video beats every photo or text post.

The AI trend feedback path is post-selection context in the checked revision.
Do not turn it into a local “trend boost” without new evidence.

Always separate:

- `confirmed`: directly present in FxTwitter/source data
- `inferred`: deterministic or model-supported interpretation
- `unknown`: unavailable, blocked or not proven
