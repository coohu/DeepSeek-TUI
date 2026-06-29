Feature: Tool call lifecycle
  Scenario: Happy path lists the current directory through a tool
    # This executable slice asserts the public exec stream and mocked LLM border.
    # The PTY screen slice should also assert Statusline state and BlueWhale activity:
    # running while the tool is executing, stopped or completed when the turn finishes.
    Given an offline DeepSeek workspace containing:
      | path      | kind   |
      | README.md | file   |
      | notes.txt | file   |
      | src       | folder |
    And the mocked LLM will request the "list_dir" tool with:
      | path |
      | .    |
    And the mocked LLM will answer after the tool result:
      | content                                                    |
      | The directory contains README.md, notes.txt, and src/.      |
    When the user asks "list the current directory"
    Then DeepSeek should send the user request to the mocked LLM
    And the public tool lifecycle should show a running tool:
      | status  | marker | tool     | input |
      | running | [~]    | list_dir | .     |
    And the public tool result should return directory entries:
      | entry     | kind   |
      | README.md | file   |
      | notes.txt | file   |
      | src       | folder |
    And DeepSeek should send the tool result back to the mocked LLM
    And the public tool lifecycle should show a completed tool:
      | status    | marker | tool     | input |
      | completed | ✓      | list_dir | .     |
    And the public output should include "The directory contains README.md, notes.txt, and src/."

  Scenario: Unknown tool returns an error result
    Given an offline DeepSeek workspace containing:
      | path      | kind |
      | README.md | file |
    And the mocked LLM will request the "missing_tool" tool with:
      | path |
      | .    |
    And the mocked LLM will answer after the tool result:
      | content                                      |
      | I could not run the requested missing tool. |
    When the user asks "try a missing tool"
    Then DeepSeek should send the user request to the mocked LLM
    And the public tool lifecycle should show a running tool:
      | status  | marker | tool         | input |
      | running | [~]    | missing_tool | .     |
    And the public tool result should report an error for "missing_tool"
    And DeepSeek should send the tool error back to the mocked LLM
    And the public tool lifecycle should show a failed tool:
      | status | marker | tool         | input |
      | error  | [!]    | missing_tool | .     |
    And the public output should include "I could not run the requested missing tool."

  Scenario: Malformed tool arguments return an error result
    Given an offline DeepSeek workspace containing:
      | path      | kind |
      | README.md | file |
    And the mocked LLM will request the "list_dir" tool with malformed arguments "{not-json"
    And the mocked LLM will answer after the tool result:
      | content                                 |
      | I could not parse the tool arguments. |
    When the user asks "try malformed tool arguments"
    Then DeepSeek should send the user request to the mocked LLM
    And the public tool lifecycle should show a running tool with raw input for "list_dir"
    And the public tool result should report malformed arguments for "list_dir"
    And DeepSeek should send the malformed argument error back to the mocked LLM
    And the public tool lifecycle should show a failed tool with raw input for "list_dir"
    And the public output should include "I could not parse the tool arguments."
