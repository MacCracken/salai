# Benchmarks

Three-point tracking: **baseline** (first run) / **previous** / **latest**

| Point | Date | Commit |
|-------|------|--------|
| Baseline | 2026-03-26T09:54:44Z | `ea28ea1` |
| Previous | 2026-03-26T10:09:12Z | `ea28ea1` |
| Latest | 2026-03-26T11:07:41Z | `049fc7e` |

## editor_app_new

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `editor_app_new` | 111.65 ns | 88.75 ns | 92.78 ns |

## editor_state_toggle_play

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `editor_state_toggle_play` | 293.60 ps | 261.70 ps | 296.70 ps |

## editor_step_frame

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `editor_step_frame` | 14.02 ns | 13.36 ns | 14.61 ns |

## editor_state_serde_roundtrip

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `editor_state_serde_roundtrip` | 658.42 ns | 315.97 ns | 344.16 ns |

## inspect_empty_entity

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `inspect_empty_entity` | 34.70 ns | 17.58 ns | 19.51 ns |

## inspect_entity_5_components

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `inspect_entity_5_components` | 2.01 µs | 932.10 ns | 980.12 ns |

## hierarchy_flat_10

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `hierarchy_flat_10` | 1.09 µs | 561.61 ns | 582.75 ns |

## hierarchy_flat_100

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `hierarchy_flat_100` | 8.56 µs | 6.76 µs | 6.62 µs |

## hierarchy_deep_chain_20

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `hierarchy_deep_chain_20` | 1.96 µs | 1.60 µs | 1.75 µs |

## hierarchy_wide_50_children

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `hierarchy_wide_50_children` | 4.61 µs | 3.76 µs | 4.26 µs |

## flatten_hierarchy_51_nodes

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `flatten_hierarchy_51_nodes` | 330.51 ns | 316.10 ns | 395.09 ns |

## viewport_default

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `viewport_default` | 13.56 ns | 12.93 ns | 14.45 ns |

## viewport_rotate

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `viewport_rotate` | 18.45 ns | 17.24 ns | 18.95 ns |

## viewport_zoom

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `viewport_zoom` | 9.45 ns | 9.30 ns | 9.31 ns |

## viewport_cycle_gizmo

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `viewport_cycle_gizmo` | 528.00 ps | 515.70 ps | 521.20 ps |

## eval_simple_arithmetic

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `eval_simple_arithmetic` | 167.07 ns | 157.55 ns | 165.26 ns |

## eval_complex_expression

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `eval_complex_expression` | 626.52 ns | 591.96 ns | 615.55 ns |

## eval_plain_number

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `eval_plain_number` | 68.08 ns | 62.23 ns | 66.64 ns |

## eval_or_fallback

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `eval_or_fallback` | 69.26 ns | 66.16 ns | 74.84 ns |

## eval_or_parse_number

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `eval_or_parse_number` | 68.88 ns | 64.28 ns | 71.86 ns |

## hw_detect

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `hw_detect` | 1.00 ms | 1.00 ms | 1.01 ms |

## history_record

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `history_record` | 1.16 µs | 1.12 µs | 1.16 µs |

## history_undo_redo_cycle

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `history_undo_redo_cycle` | 6.02 ns | 5.53 ns | 5.90 ns |

## history_verify_100_entries

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `history_verify_100_entries` | 46.38 µs | 44.30 µs | 41.85 µs |

## npc_personality_new

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `npc_personality_new` | 19.57 ns | 18.29 ns | 19.30 ns |

## npc_inspector_summary

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `npc_inspector_summary` | 78.02 ns | 90.44 ns | 93.39 ns |

## npc_compatibility

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `npc_compatibility` | 16.77 ns | 16.10 ns | 16.87 ns |

## npc_blend

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `npc_blend` | 115.51 ns | 111.37 ns | 120.28 ns |

## texture_average_color_256x256

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `texture_average_color_256x256` | 28.50 µs | 26.55 µs | 27.94 µs |

## texture_thumbnail_512_to_128

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `texture_thumbnail_512_to_128` | 368.91 µs | 363.66 µs | 413.00 µs |

## texture_inspect_64x64

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `texture_inspect_64x64` | 1.70 µs | 2.05 µs | 1.85 µs |

## audio_inspect_1sec

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `audio_inspect_1sec` | 9.52 µs | 11.59 µs | 9.59 µs |

## audio_waveform_1sec_100pps

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `audio_waveform_1sec_100pps` | 146.02 µs | 150.28 µs | 148.70 µs |

## scene_add_entity

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `scene_add_entity` | 824.01 ns | 835.96 ns | 843.50 ns |

## scene_extract_50_entities

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `scene_extract_50_entities` | 6.57 µs | 6.11 µs | 6.03 µs |

## scene_to_toml_10_entities

| Benchmark | Baseline | Previous | Latest |
|-----------|----------|----------|--------|
| `scene_to_toml_10_entities` | 23.54 µs | 20.80 µs | 21.42 µs |

---

Generated by `./scripts/bench-history.sh`. Full history in `bench-history.csv`.
