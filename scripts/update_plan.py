import os

from dotenv import load_dotenv

load_dotenv()
from openhands.sdk import LLM, Agent, Conversation, Tool
from openhands.tools.file_editor import FileEditorTool
from openhands.tools.task_tracker import TaskTrackerTool
from openhands.tools.terminal import TerminalTool

llm = LLM(
    model=os.getenv("LLM_MODEL", "anthropic/glm-4.7"),
    api_key=os.getenv("ZAI_API_KEY"),
    base_url=os.getenv("ZAI_BASE_URL", None),
)

agent = Agent(
    llm=llm,
    tools=[
        Tool(name=TerminalTool.name),
        Tool(name=FileEditorTool.name),
        Tool(name=TaskTrackerTool.name),
    ],
)

cwd = os.path.dirname(os.getcwd())
conversation = Conversation(agent=agent, workspace=cwd)


status_update_prompt = """STATUS UPDATE:

The planning phase is COMPLETE. The task list has been created with 15 tasks organized across 4 phases.

CURRENT STATE:
- Phase 1 (Planning): ✓ DONE
- Task list created with all phases (0.1-0.3, 1.1-1.3, 2.1-2.3, 3.1-3.3, 4.1-4.3)
- All tasks currently marked as "todo"

NEXT STEPS:
1. Update CROW_AGENT_PLAN.md to reflect that planning is complete
2. Mark Phase 0.1 as "in_progress"
3. Begin Phase 0.1: Integrate OpenHands as default ACP agent in crow_ide

Please update the plan document and begin implementation.
"""

conversation.send_message(status_update_prompt)
conversation.run()
print("All done!")
