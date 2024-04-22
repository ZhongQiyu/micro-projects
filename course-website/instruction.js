// instruction.js

// 封装控制电机转动的指令
function controlMotor(direction, speed) {
  // 构建指令
  const instruction = {
    type: 'motor_control',
    direction: direction,
    speed: speed
  };

  // 返回封装好的指令
  return instruction;
}

// 导出指令函数
module.exports = {
  controlMotor
};

// 在小程序中使用封装的指令
const instructionModule = require('./instruction.js');

// 获取控制电机转动的指令
const motorInstruction = instructionModule.controlMotor('forward', 50);

// 将指令发送给边缘服务器
sendInstructionToServer(motorInstruction);
