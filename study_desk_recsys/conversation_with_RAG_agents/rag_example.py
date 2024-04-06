# -*- coding: utf-8 -*-
"""
A simple example for conversation between user and
an agent with RAG capability.
"""
import json
import os
import imggen

from rag_agents import LlamaIndexAgent

import agentscope
from agentscope.agents import UserAgent
from agentscope.message import Msg
from agentscope.agents import DialogAgent


def main() -> None:
    """A RAG multi-agent demo"""
    agentscope.init(
        model_configs=[
            {
                "model_type": "dashscope_chat",
                "config_name": "qwen_config",
                "model_name": "qwen-max",
                "api_key":f"{os.environ.get('DASHSCOPE_API_KEY')}",
            },
            {
                "model_type": "dashscope_text_embedding",
                "config_name": "qwen_emb_config",
                "model_name": "text-embedding-v2",
                "api_key": f"{os.environ.get('DASHSCOPE_API_KEY')}",
            },

        ],
    )

    with open("./agent_config.json", "r", encoding="utf-8") as f:
        agent_configs = json.load(f)
    #tutorial_agent = LlamaIndexAgent(**agent_configs[0]["args"])
    #code_explain_agent = LlamaIndexAgent(**agent_configs[1]["args"])
    summarize_agent = DialogAgent(**agent_configs[2]["args"])
    tutor_sale_agent = DialogAgent(**agent_configs[3]["args"])
    extract_agent = DialogAgent(**agent_configs[4]["args"])

    user_agent = UserAgent()
    message = ""
    i = 0
    # start the conversation between user and assistant
    while True:
        x = user_agent()
        i = i+1
        x.role = "user"  # to enforce dashscope requirement on roles
        if len(x["content"]) == 0 or str(x["content"]).startswith("exit"):
            break
        tutorial_response = tutor_sale_agent(x)
        #code_explain = code_explain_agent(x)
        msg = Msg(
            name="user",
            role="user",
            content=tutorial_response["content"]
            + "\n"
            + "----------------------------------------------"
        #    + code_explain["content"]
            + "\n"
            + x["content"],
        )
        message = message+msg.content



    if i > 1 :

        msg2= Msg(
                name="user",
                role="user",
                content=message
                + "\n"
                + "----------------------------------------------"
            #    + code_explain["content"]
                + "\n"
                + x["content"],
            )
        message2 = summarize_agent(msg2)
        finalprompt = extract_agent(message2)
    else:
        finalprompt = "default"
    imggen.createimg(finalprompt)



if __name__ == "__main__":
    main()
