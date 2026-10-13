# 📜 Structured LLM XML Context Schemas

Cartograph emits unambiguous structured XML tags, establishing clear semantic boundaries that prevent LLM hallucinations.

## 1. `<cartograph_context>` (`locate`)

```xml
<cartograph_context query="authenticate" confidence="0.98">
  <target id="auth::authenticate" file="src/auth.rs" line="42" col="1" kind="function">
    <signature>pub fn authenticate(user: &amp;str, token: &amp;str) -&gt; Result&lt;Session&gt;</signature>
    <body>... full body snippet ...</body>
  </target>
  <callers count="2">
    <node file="src/routes/login.rs" line="18" symbol="login_handler" signature="fn login_handler(...)"/>
  </callers>
</cartograph_context>
```

## 2. `<blast_radius_analysis>` (`impact`)

```xml
<blast_radius_analysis root="auth::authenticate" risk_score="0.82" breaking="true">
  <affected_dependents count="5">
    <node file="src/routes/login.rs" line="18" symbol="login_handler" impact="direct"/>
  </affected_dependents>
</blast_radius_analysis>
```

## 3. `<verification_gate>` (`verify`)

```xml
<verification_gate status="PASSED" duration_ms="185">
  <check name="ast_syntax" status="OK"/>
  <check name="architectural_boundaries" status="OK"/>
  <check name="breaking_changes" status="OK"/>
  <check name="predictive_tests" passed="4" failed="0"/>
</verification_gate>
```
