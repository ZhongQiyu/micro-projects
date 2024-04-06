# -*- coding: utf-8 -*-
"""A simple example for conversation between user and assistant agent."""
import agentscope
from agentscope.agents import DialogAgent
from agentscope.agents.user_agent import UserAgent


import dashscope
import textwrap
from dashscope import Generation

import requests
from PIL import Image
from dashscope import ImageSynthesis

import os
from http import HTTPStatus


def createimg(description):
    """A basic conversation demo"""
    MY_API_KEY = f"{os.environ.get('DASHSCOPE_API_KEY')}"
    agentscope.init(
        model_configs=[
            {
                "config_name": "my_dashscope_image_synthesis_config",
                "model_type": "dashscope_image_synthesis",

                # Required parameters
                "model_name": "wanx-v1",  # DashScope Image Synthesis API model name, e.g., wanx-v1

                # Optional parameters
                "api_key": MY_API_KEY,
                "generate_args": {
                    "n": 1,
                    "size": "1024*1024"
                    # ...
                }
            },

            {
                "config_name": "my_dashscope_chat_config",
                "model_type": "dashscope_chat",

                # Required parameters
                "model_name": "qwen-max",               # DashScope Chat API model name, e.g., qwen-max

                # Optional parameters
                "api_key": MY_API_KEY,                # DashScope API Key, will be read from environment variables if not provided
                "generate_args": {
                    # e.g., "temperature": 0.5
                },
            },
        ],
    )

    # Init two agents
    INIT_PROMT = "You are a senior product manager, skilled in product design and creating visual prototypes of product appearances."
    dialog_agent = DialogAgent(
        name="Design Assistant",
        sys_prompt=INIT_PROMT,
        model_config_name="my_dashscope_chat_config",  # replace by your model config name
    )
    user_agent = UserAgent()

    # start the conversation between user and assistant

    ### Image generation code ###
    LLM_model = 'qwen-max'
    dashscope.api_key = MY_API_KEY
#    x = user_agent()
#    print("用户输入：",x)

    img_file = 'img.jpg'
    if description == "default":
        print("default value used:白色地中海")
        description = "白色地中海风格"


    instruction = f'''
        generate an english detailed prompt to be used for text to image generation for product. the original prompt is in```
        ```
            一套儿童书桌{description}
        ```
        please return prompt only, less than 100 description, nothing else.
        '''
#        response = Generation.call(
#            model=LLM_model,
#            prompt=instruction
#        )
#        text2image_prompt = response.output['text']
#        print(textwrap.fill(text2image_prompt, width=80))
#        print("instruction is now", instruction)
    dialog_agent.sys_prompt = instruction
    print("提示词现在是：", dialog_agent.sys_prompt)

    x = dialog_agent()
    generate_img_file(x.content,img_file)
#        print("x.content-dialog 现在是：", x.content)




#generate_img_file(text2image_prompt, img_file)


def generate_img_file(desc, img_file):
    from dashscope.common.error import InvalidTask
    dashscope.api_key = os.environ.get("DASHSCOPE_API_KEY")
    assert dashscope.api_key

    prompt = desc
    try:
        print("正在进行图片生成，请稍后。。。")
        rsp = dashscope.ImageSynthesis.call(
            model='wanx-v1',
            prompt=prompt,
            n=1,
            size='1024*1024')
        print("正在写入图片。。。")
        # save file to current directory
        if rsp.status_code == HTTPStatus.OK:
            for result in rsp.output.results:
                with open(img_file, 'wb+') as f:
                    f.write(requests.get(result.url).content)
                    img = Image.open(img_file)
                    img.show()
        else:
            print('Failed, status_code: %s, code: %s, message: %s' %
                  (rsp.status_code, rsp.code, rsp.message))
    except InvalidTask as e:
        print(e)



