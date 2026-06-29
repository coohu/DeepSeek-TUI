@long-running
# [LONG RUNNING] Plugin discovery and listing acceptance tests. Run with:
# cargo test -p deepseek-tui --bin deepseek-tui --features long-running-tests plugin_e2e_acceptance -- --test-threads=1
Feature: Plugin discovery and listing

  Scenario: Plugin scripts are discovered from the configured plugin directory
    Given an offline DeepSeek workspace with a configured plugin directory
    And the plugin directory contains:
      | name       | description                | approval |
      | greet      | Say hello to the user      | auto     |
      | audit      | Run a security audit       | required |
      | summarizer | Summarize the given input  | suggest  |
    When the plugin scanner discovers plugins
    Then the scanner should report 3 plugins
    And the scanned plugin "greet" should have "Say hello to the user" as description
    And the scanned plugin "greet" should have "auto" as approval
    And the scanned plugin "audit" should have "required" as approval
    And the scanned plugin "summarizer" should have "suggest" as approval
    And the scanned plugin "missing-plugin" should not be found

  Scenario: Empty plugin directory reports no plugins
    Given an offline DeepSeek workspace with a configured plugin directory
    And the plugin directory is empty
    When the plugin scanner discovers plugins
    Then the scanner should report 0 plugins

  Scenario: Missing plugin directory reports the path
    Given an offline DeepSeek workspace with a configured plugin directory
    And the plugin directory does not exist
    When the plugin scanner runs
    Then the scanner should report the missing directory path
