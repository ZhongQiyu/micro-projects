# dgl_test.py
import dgl
import torch

# 边列表，表示两个节点之间的边
src = torch.tensor([0, 1, 2, 3, 4])
dst = torch.tensor([1, 2, 3, 4, 0])

# 创建DGL图
g = dgl.graph((src, dst))

# 为节点分配特征
g.ndata['feat'] = torch.randn(g.num_nodes(), 10)  # 假设每个节点有10个特征

# 为边分配特征
g.edata['weight'] = torch.randn(g.num_edges(), 1)  # 假设每条边有1个特征



import dgl.nn as dglnn
import torch.nn as nn

class GraphConvModel(nn.Module):
    def __init__(self):
        super(GraphConvModel, self).__init__()
        self.conv1 = dglnn.GraphConv(10, 32)  # 10个输入特征到32个输出特征
        self.conv2 = dglnn.GraphConv(32, 16)  # 32个输入特征到16个输出特征

    def forward(self, g, features):
        x = self.conv1(g, features)
        x = torch.relu(x)
        x = self.conv2(g, x)
        return x

model = GraphConvModel()
features = g.ndata['feat']
labels = ...  # 真实标签
train_mask = ...  # 一个布尔张量，表示哪些节点用于训练

loss_fn = nn.CrossEntropyLoss()
optimizer = torch.optim.Adam(model.parameters(), lr=0.01)

for epoch in range(epochs):
    model.train()
    logits = model(g, features)
    loss = loss_fn(logits[train_mask], labels[train_mask])

    optimizer.zero_grad()
    loss.backward()
    optimizer.step()



import torch
if torch.backends.mps.is_available():
    mps_device = torch.device("mps")
    x = torch.ones(1, device=mps_device)
    print (x)
else:
    print ("MPS device not found.")