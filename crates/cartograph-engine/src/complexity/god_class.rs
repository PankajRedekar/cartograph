use cartograph_ast::ClassComplexity;
use cartograph_core::config::ComplexityConfig;
use cartograph_core::model::{
    ClassCohesionMetric, DecompositionCluster, GodClassEvaluation, GodClassSeverity,
};
use std::collections::{HashMap, HashSet, VecDeque};

/// Method execution profile within a class
#[derive(Debug, Clone)]
pub struct MethodProfile {
    pub name: String,
    pub cyclomatic: u32,
    pub fields_accessed: HashSet<String>,
    pub methods_called: HashSet<String>,
}

pub struct GodClassAnalyzer;

impl GodClassAnalyzer {
    /// Compute LCOM4 connected components, cohesion metrics, and God-Class evaluation
    pub fn evaluate(
        class: &ClassComplexity,
        methods: &[MethodProfile],
        config: &ComplexityConfig,
    ) -> (ClassCohesionMetric, GodClassEvaluation) {
        let method_count = methods.len();
        let field_count = class.fields.len();
        let wmc = class.wmc;

        if method_count <= 1 {
            let cohesion = ClassCohesionMetric {
                wmc,
                method_count,
                field_count,
                lcom4: 1,
                tcc: 1.0,
                atfd: 0,
            };
            let eval = GodClassEvaluation {
                is_god_class: false,
                severity: Some(GodClassSeverity::Clean),
                god_class_index: 0.0,
                triggers: Vec::new(),
                candidate_extractions: Vec::new(),
            };
            return (cohesion, eval);
        }

        // Build undirected graph over methods
        // Edge exists between method i and j if:
        // 1. Shared field access: F(i) ∩ F(j) ≠ ∅
        // 2. Direct call: i calls j OR j calls i
        let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut connected_pairs = 0usize;

        for i in 0..method_count {
            for j in (i + 1)..method_count {
                let m_i = &methods[i];
                let m_j = &methods[j];

                let shares_field = m_i
                    .fields_accessed
                    .intersection(&m_j.fields_accessed)
                    .next()
                    .is_some();

                let direct_call = m_i.methods_called.contains(&m_j.name)
                    || m_j.methods_called.contains(&m_i.name);

                if shares_field || direct_call {
                    adj.entry(i).or_default().push(j);
                    adj.entry(j).or_default().push(i);
                    connected_pairs += 1;
                }
            }
        }

        // Compute connected components via BFS
        let mut visited = vec![false; method_count];
        let mut components: Vec<Vec<usize>> = Vec::new();

        for i in 0..method_count {
            if !visited[i] {
                let mut comp = Vec::new();
                let mut queue = VecDeque::new();
                visited[i] = true;
                queue.push_back(i);

                while let Some(curr) = queue.pop_front() {
                    comp.push(curr);
                    if let Some(neighbors) = adj.get(&curr) {
                        for &next in neighbors {
                            if !visited[next] {
                                visited[next] = true;
                                queue.push_back(next);
                            }
                        }
                    }
                }
                components.push(comp);
            }
        }

        let lcom4 = components.len().max(1);
        let total_possible_pairs = (method_count * (method_count - 1)) / 2;
        let tcc = if total_possible_pairs > 0 {
            connected_pairs as f64 / total_possible_pairs as f64
        } else {
            1.0
        };

        let cohesion = ClassCohesionMetric {
            wmc,
            method_count,
            field_count,
            lcom4,
            tcc,
            atfd: 0,
        };

        // Evaluate God-Class heuristics
        let mut triggers = Vec::new();
        if wmc >= config.max_class_wmc {
            triggers.push(format!(
                "High WMC (Weighted Methods per Class = {}) exceeds threshold ({})",
                wmc, config.max_class_wmc
            ));
        }
        if class.loc >= config.max_class_loc {
            triggers.push(format!(
                "Class LOC ({}) exceeds threshold ({})",
                class.loc, config.max_class_loc
            ));
        }
        if method_count >= config.max_class_methods {
            triggers.push(format!(
                "Method count ({}) exceeds threshold ({})",
                method_count, config.max_class_methods
            ));
        }
        if lcom4 >= config.min_lcom4 {
            triggers.push(format!(
                "High LCOM4 ({}) indicates {} disconnected responsibilities",
                lcom4, lcom4
            ));
        }
        if tcc < 0.33 && method_count >= 5 {
            triggers.push(format!(
                "Low Tight Class Cohesion ({:.1}%) indicates loosely coupled methods",
                tcc * 100.0
            ));
        }

        // Compute God-Class Index (0 - 100)
        let wmc_factor = (wmc as f64 / config.max_class_wmc as f64).min(2.0) * 30.0;
        let loc_factor = (class.loc as f64 / config.max_class_loc as f64).min(2.0) * 25.0;
        let method_factor = (method_count as f64 / config.max_class_methods as f64).min(2.0) * 20.0;
        let lcom4_factor = ((lcom4 as f64 - 1.0) / 3.0).clamp(0.0, 1.0) * 25.0;

        let god_class_index =
            (wmc_factor + loc_factor + method_factor + lcom4_factor).clamp(0.0, 100.0);

        let is_god_class = triggers.len() >= 2 || god_class_index >= config.max_god_class_index;

        let severity = if !is_god_class {
            Some(GodClassSeverity::Clean)
        } else if god_class_index >= 80.0 || triggers.len() >= 4 {
            Some(GodClassSeverity::Critical)
        } else if god_class_index >= 65.0 || triggers.len() >= 3 {
            Some(GodClassSeverity::Major)
        } else {
            Some(GodClassSeverity::Minor)
        };

        // Synthesize decomposition clusters if LCOM4 > 1
        let mut candidate_extractions = Vec::new();
        if lcom4 > 1 {
            for (idx, comp) in components.iter().enumerate() {
                let comp_methods: Vec<String> =
                    comp.iter().map(|&i| methods[i].name.clone()).collect();
                let mut comp_fields = HashSet::new();
                for &i in comp {
                    for f in &methods[i].fields_accessed {
                        comp_fields.insert(f.clone());
                    }
                }

                let method_pct = (comp.len() as f64 / method_count as f64) * 100.0;
                let candidate_name = synthesize_cluster_name(&class.name, &comp_methods, idx + 1);

                candidate_extractions.push(DecompositionCluster {
                    candidate_name,
                    methods: comp_methods,
                    fields: comp_fields.into_iter().collect(),
                    method_percentage: method_pct,
                    rationale: format!(
                        "Cluster {} isolates {} methods with internal cohesion, decoupled from other class members.",
                        idx + 1,
                        comp.len()
                    ),
                });
            }
        }

        let eval = GodClassEvaluation {
            is_god_class,
            severity,
            god_class_index,
            triggers,
            candidate_extractions,
        };

        (cohesion, eval)
    }

    /// Generate AI Refactoring Blueprint prompt for an autonomous coding agent
    pub fn generate_refactor_prompt(
        class_name: &str,
        filepath: &str,
        eval: &GodClassEvaluation,
        cohesion: &ClassCohesionMetric,
    ) -> String {
        let mut prompt = String::new();
        prompt.push_str(&format!(
            "# AI Refactoring Blueprint: Decomposition of God-Class `{}`\n\n",
            class_name
        ));
        prompt.push_str(&format!("**Filepath**: `{}`\n", filepath));
        prompt.push_str(&format!(
            "**Metrics**: WMC: {}, Methods: {}, Fields: {}, LCOM4: {}, GCI: {:.1} ({:?})\n\n",
            cohesion.wmc,
            cohesion.method_count,
            cohesion.field_count,
            cohesion.lcom4,
            eval.god_class_index,
            eval.severity.unwrap_or(GodClassSeverity::Clean)
        ));

        prompt.push_str("## Architectural Problem\n");
        for trigger in &eval.triggers {
            prompt.push_str(&format!("- {}\n", trigger));
        }
        prompt.push('\n');

        prompt.push_str("## Decomposition Plan (LCOM4 Connected Components)\n");
        for cluster in &eval.candidate_extractions {
            prompt.push_str(&format!(
                "### Extract Component: `{}` ({:.1}% of methods)\n",
                cluster.candidate_name, cluster.method_percentage
            ));
            prompt.push_str(&format!("- **Rationale**: {}\n", cluster.rationale));
            prompt.push_str(&format!(
                "- **Methods to Move**: `{}`\n",
                cluster.methods.join("`, `")
            ));
            if !cluster.fields.is_empty() {
                prompt.push_str(&format!(
                    "- **Fields to Encapsulate**: `{}`\n",
                    cluster.fields.join("`, `")
                ));
            }
            prompt.push('\n');
        }

        prompt.push_str("## Refactoring Instructions for Autonomous AI Agent\n");
        prompt.push_str(&format!(
            "1. Create dedicated files/structs for each extracted component candidate.\n\
             2. Transfer the specified methods and private fields into the new components.\n\
             3. In `{class_name}`, replace extracted logic with composition: delegate calls to injected instances.\n\
             4. Preserve exact backward compatibility for all existing public APIs.\n\
             5. Run unit tests and verify zero regressions.\n"
        ));

        prompt
    }
}

fn synthesize_cluster_name(class_name: &str, methods: &[String], cluster_id: usize) -> String {
    // Find common prefixes or keywords among methods
    let keywords = [
        "auth", "token", "session", "user", "order", "payment", "cache", "event", "log", "metric",
        "render", "config", "http", "db",
    ];
    for kw in keywords {
        if methods.iter().any(|m| m.to_lowercase().contains(kw)) {
            let capitalized = format!("{}{}", kw[..1].to_uppercase(), &kw[1..]);
            return format!("{class_name}{capitalized}Handler");
        }
    }

    if let Some(first_method) = methods.first() {
        let parts: Vec<&str> = first_method.split('_').collect();
        if let Some(lead) = parts.first() {
            let capitalized = format!("{}{}", lead[..1].to_uppercase(), &lead[1..]);
            return format!("{class_name}{capitalized}Subsystem");
        }
    }

    format!("{class_name}Part{cluster_id}")
}
