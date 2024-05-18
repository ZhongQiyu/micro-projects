%% 数据读取与预处理
trafficData = readtable('traffic_data.csv');
demandData = readtable('demand_data.csv');

%% 网络构建与需求分配
% 构建带有权重属性的图
G = graph(trafficData.StartNode, trafficData.EndNode, trafficData.Weight);

ODMatrix = zeros(numnodes(G), numnodes(G));
for i = 1:height(demandData)
    ODMatrix(demandData.StartNode(i), demandData.EndNode(i)) = demandData.Demand(i);
end

%% 参数调整与优化策略模拟
% 定义几种优化策略
strategies = {'IncreaseCapacity', 'DecreaseDemand'};

% 策略参数范围
paramRange = 0.8:0.1:1.2;

% 初始化结果存储结构
resultsTable = table();

% 遍历每种策略和参数
for strategy = strategies
    totalTravelTimes = zeros(size(paramRange));  % 存储每个策略的总旅行时间
    for paramIndex = 1:length(paramRange)
        param = paramRange(paramIndex);
        
        % 应用策略
        [adjustedG, adjustedODMatrix] = applyStrategy(G, ODMatrix, strategy{1}, param);

        % 交通流分配 - 用户均衡（UE）
        [flow, travelTime] = userEquilibrium(adjustedG, adjustedODMatrix);

        % 结果分析
        totalTravelTime = sum(travelTime);
        totalTravelTimes(paramIndex) = totalTravelTime;
        
        % 将结果添加到表格中
        newRow = table(strategy, param, totalTravelTime);
        resultsTable = [resultsTable; newRow];
        
        % 绘制当前策略下不同参数对总旅行时间的影响
        figure;  % 每次循环创建一个新的图形
        plot(paramRange, totalTravelTimes, 'DisplayName', strategy{1});
        xlabel('参数值');
        ylabel('总旅行时间 (分钟)');
        title(['不同策略和参数下的总旅行时间 - ', strategy{1}]);
        legend();
    end
end

% 定义结果表格的列名
resultsTable.Properties.VariableNames = {'Strategy', 'Parameter', 'TotalTravelTime'};

% 显示结果表格
disp(resultsTable);

% 结果展示
for row = 1:height(resultsTable)
    fprintf('策略: %s, 参数: %.1f, 总旅行时间: %.2f 分钟\n', ...
        resultsTable.Strategy{row}, resultsTable.Parameter(row), resultsTable.TotalTravelTime(row));
end

%% 函数定义
function [adjustedG, adjustedODMatrix] = applyStrategy(G, ODMatrix, strategy, param)
    adjustedG = G;
    adjustedODMatrix = ODMatrix;
    switch strategy
        case 'IncreaseCapacity'
            % 增加道路容量
            adjustedG.Edges.Weight = G.Edges.Weight * param;
        case 'DecreaseDemand'
            % 减少需求
            adjustedODMatrix = ODMatrix * param;
    end
end

function [flow, travelTime] = userEquilibrium(G, ODMatrix)
    flow = zeros(size(G.Edges, 1), 1);
    for origin = 1:size(ODMatrix, 1)
        for destination = 1:size(ODMatrix, 2)
            if ODMatrix(origin, destination) > 0
                [path, ~] = shortestpath(G, origin, destination);
                for p = 1:(length(path)-1)
                    edgeIndex = findedge(G, path(p), path(p+1));
                    flow(edgeIndex) = flow(edgeIndex) + ODMatrix(origin, destination);
                end
            end
        end
    end
    travelTime = flow / 100; % 假设旅行时间与流量成正比
end