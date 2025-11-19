const { useState, useEffect } = React;

const DispatcherDemo = () => {
  const [orders, setOrders] = useState([]);
  const [drivers, setDrivers] = useState([
    { 
      id: 1, 
      name: '张师傅', 
      status: 'idle', 
      capacity: 3, 
      currentLoad: 0, 
      vehicleType: '电动车',
      position: { lat: 39.9042, lng: 116.4074 },
      rating: 4.8,
      completedOrders: 156
    },
    { 
      id: 2, 
      name: '李师傅', 
      status: 'idle', 
      capacity: 2, 
      currentLoad: 0, 
      vehicleType: '摩托车',
      position: { lat: 39.9100, lng: 116.4100 },
      rating: 4.9,
      completedOrders: 203
    },
    { 
      id: 3, 
      name: '王师傅', 
      status: 'idle', 
      capacity: 4, 
      currentLoad: 0, 
      vehicleType: '汽车',
      position: { lat: 39.9000, lng: 116.4000 },
      rating: 4.7,
      completedOrders: 189
    },
    { 
      id: 4, 
      name: '赵师傅', 
      status: 'idle', 
      capacity: 3, 
      currentLoad: 0, 
      vehicleType: '电动车',
      position: { lat: 39.9080, lng: 116.4120 },
      rating: 4.6,
      completedOrders: 145
    }
  ]);
  const [isRunning, setIsRunning] = useState(false);
  const [strategy, setStrategy] = useState('nearest_first');
  const [stats, setStats] = useState({
    totalOrders: 0,
    dispatched: 0,
    pending: 0,
    avgDistance: 0,
    avgDispatchTime: 0
  });

  const strategies = {
    nearest_first: '就近派单',
    load_balanced: '负载均衡',
    rating_priority: '高评分优先',
    capacity_match: '容量匹配'
  };

  const orderTypes = [
    { type: '外卖订单', icon: '🍔', distance: () => Math.random() * 5 + 0.5 },
    { type: '快递订单', icon: '📦', distance: () => Math.random() * 10 + 1 },
    { type: '生鲜订单', icon: '🥬', distance: () => Math.random() * 3 + 0.5 },
    { type: '打车订单', icon: '🚗', distance: () => Math.random() * 15 + 2 }
  ];

  const generateOrder = () => {
    const orderType = orderTypes[Math.floor(Math.random() * orderTypes.length)];
    const distance = parseFloat(orderType.distance().toFixed(2));
    const price = (distance * 5 + Math.random() * 20).toFixed(2);
    
    return {
      id: Date.now() + Math.random(),
      type: orderType.type,
      icon: orderType.icon,
      status: 'pending',
      distance: distance,
      price: parseFloat(price),
      pickupLocation: { 
        lat: 39.9042 + (Math.random() - 0.5) * 0.02, 
        lng: 116.4074 + (Math.random() - 0.5) * 0.02 
      },
      deliveryLocation: { 
        lat: 39.9042 + (Math.random() - 0.5) * 0.02, 
        lng: 116.4074 + (Math.random() - 0.5) * 0.02 
      },
      createTime: new Date().toLocaleTimeString(),
      estimatedTime: Math.ceil(distance * 3 + Math.random() * 10),
      assignedTo: null
    };
  };

  const calculateDistance = (pos1, pos2) => {
    const R = 6371;
    const dLat = (pos2.lat - pos1.lat) * Math.PI / 180;
    const dLng = (pos2.lng - pos1.lng) * Math.PI / 180;
    const a = Math.sin(dLat/2) * Math.sin(dLat/2) +
              Math.cos(pos1.lat * Math.PI / 180) * Math.cos(pos2.lat * Math.PI / 180) *
              Math.sin(dLng/2) * Math.sin(dLng/2);
    const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1-a));
    return R * c;
  };

  const dispatchOrder = (order) => {
    let selectedDriver = null;

    const availableDrivers = drivers.filter(d => d.currentLoad < d.capacity);
    
    if (availableDrivers.length === 0) return order;

    switch(strategy) {
      case 'nearest_first':
        selectedDriver = availableDrivers.reduce((nearest, driver) => {
          const driverDist = calculateDistance(driver.position, order.pickupLocation);
          const nearestDist = calculateDistance(nearest.position, order.pickupLocation);
          return driverDist < nearestDist ? driver : nearest;
        });
        break;

      case 'load_balanced':
        selectedDriver = availableDrivers.sort((a, b) => 
          (a.currentLoad / a.capacity) - (b.currentLoad / b.capacity)
        )[0];
        break;

      case 'rating_priority':
        selectedDriver = availableDrivers.sort((a, b) => {
          const ratingDiff = b.rating - a.rating;
          if (Math.abs(ratingDiff) > 0.1) return ratingDiff;
          return (a.currentLoad / a.capacity) - (b.currentLoad / b.capacity);
        })[0];
        break;

      case 'capacity_match':
        const needsLargeCapacity = order.type === '快递订单' || order.type === '生鲜订单';
        const filtered = needsLargeCapacity 
          ? availableDrivers.filter(d => d.vehicleType === '汽车' || d.capacity >= 3)
          : availableDrivers;
        
        selectedDriver = (filtered.length > 0 ? filtered : availableDrivers).sort((a, b) => {
          const distA = calculateDistance(a.position, order.pickupLocation);
          const distB = calculateDistance(b.position, order.pickupLocation);
          return distA - distB;
        })[0];
        break;
    }

    if (selectedDriver) {
      setDrivers(prev => prev.map(d => 
        d.id === selectedDriver.id 
          ? { 
              ...d, 
              currentLoad: d.currentLoad + 1, 
              status: 'busy',
              completedOrders: d.completedOrders + 1
            }
          : d
      ));
      return { 
        ...order, 
        status: 'dispatched', 
        assignedTo: selectedDriver.name,
        driverId: selectedDriver.id 
      };
    }
    return order;
  };

  useEffect(() => {
    if (!isRunning) return;

    const interval = setInterval(() => {
      const newOrder = generateOrder();
      setOrders(prev => {
        const updated = [...prev, newOrder];
        return updated.slice(-12);
      });

      setTimeout(() => {
        setOrders(prev => {
          const pendingOrders = prev.filter(o => o.status === 'pending');
          if (pendingOrders.length > 0) {
            const orderToDispatch = pendingOrders[0];
            const dispatched = dispatchOrder(orderToDispatch);
            
            setStats(s => ({
              ...s,
              totalOrders: s.totalOrders + 1,
              dispatched: dispatched.status === 'dispatched' ? s.dispatched + 1 : s.dispatched,
              pending: prev.filter(o => o.status === 'pending').length,
              avgDistance: ((s.avgDistance * s.totalOrders + orderToDispatch.distance) / (s.totalOrders + 1)).toFixed(2)
            }));

            return prev.map(o => o.id === orderToDispatch.id ? dispatched : o);
          }
          return prev;
        });
      }, 500);

      if (Math.random() > 0.6) {
        setDrivers(prev => prev.map(d => {
          if (d.currentLoad > 0 && Math.random() > 0.4) {
            return {
              ...d,
              currentLoad: Math.max(0, d.currentLoad - 1),
              status: d.currentLoad - 1 === 0 ? 'idle' : 'busy',
              position: {
                lat: d.position.lat + (Math.random() - 0.5) * 0.005,
                lng: d.position.lng + (Math.random() - 0.5) * 0.005
              }
            };
          }
          return d;
        }));
      }
    }, 2500);

    return () => clearInterval(interval);
  }, [isRunning, strategy, drivers]);

  const reset = () => {
    setOrders([]);
    setDrivers(prev => prev.map(d => ({ 
      ...d, 
      currentLoad: 0, 
      status: 'idle',
      position: { 
        lat: 39.9042 + (Math.random() - 0.5) * 0.02, 
        lng: 116.4074 + (Math.random() - 0.5) * 0.02 
      }
    })));
    setStats({ totalOrders: 0, dispatched: 0, pending: 0, avgDistance: 0, avgDispatchTime: 0 });
  };

  const getStatusColor = (status) => {
    return status === 'dispatched' ? 'bg-green-100 text-green-800' : 'bg-blue-100 text-blue-800';
  };

  const getVehicleEmoji = (type) => {
    const emojis = {
      '电动车': '🛵',
      '摩托车': '🏍️',
      '汽车': '🚗'
    };
    return emojis[type] || '🚗';
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-orange-50 via-yellow-50 to-red-50 p-6">
      <div className="max-w-7xl mx-auto">
        <div className="bg-white rounded-2xl shadow-2xl p-8">
          <div className="flex items-center justify-between mb-8">
            <div>
              <h1 className="text-4xl font-bold text-gray-800 mb-2">🚀 Dispatcher 配送分单引擎</h1>
              <p className="text-gray-600">智能配送订单分配系统 - 实时演示</p>
            </div>
            <div className="flex gap-3">
              <button
                onClick={() => setIsRunning(!isRunning)}
                className={`px-6 py-3 rounded-lg font-semibold flex items-center gap-2 transition-all ${
                  isRunning 
                    ? 'bg-red-500 hover:bg-red-600 text-white' 
                    : 'bg-green-500 hover:bg-green-600 text-white'
                }`}
              >
                {isRunning ? '⏸ 暂停' : '▶ 开始'}
              </button>
              <button
                onClick={reset}
                className="px-6 py-3 bg-gray-500 hover:bg-gray-600 text-white rounded-lg font-semibold flex items-center gap-2 transition-all"
              >
                🔄 重置
              </button>
            </div>
          </div>

          <div className="grid grid-cols-4 gap-4 mb-8">
            <div className="bg-gradient-to-br from-blue-500 to-blue-600 rounded-xl p-6 text-white">
              <div className="flex items-center justify-between mb-2">
                <span className="text-2xl">📦</span>
                <span className="text-3xl font-bold">{stats.totalOrders}</span>
              </div>
              <p className="text-blue-100">总订单数</p>
            </div>
            <div className="bg-gradient-to-br from-green-500 to-green-600 rounded-xl p-6 text-white">
              <div className="flex items-center justify-between mb-2">
                <span className="text-2xl">📈</span>
                <span className="text-3xl font-bold">{stats.dispatched}</span>
              </div>
              <p className="text-green-100">已派送</p>
            </div>
            <div className="bg-gradient-to-br from-orange-500 to-orange-600 rounded-xl p-6 text-white">
              <div className="flex items-center justify-between mb-2">
                <span className="text-2xl">⏳</span>
                <span className="text-3xl font-bold">{stats.pending}</span>
              </div>
              <p className="text-orange-100">待派送</p>
            </div>
            <div className="bg-gradient-to-br from-purple-500 to-purple-600 rounded-xl p-6 text-white">
              <div className="flex items-center justify-between mb-2">
                <span className="text-2xl">📍</span>
                <span className="text-2xl font-semibold">{stats.avgDistance} km</span>
              </div>
              <p className="text-purple-100">平均距离</p>
            </div>
          </div>

          <div className="mb-6">
            <label className="block text-sm font-semibold text-gray-700 mb-3">选择派单策略</label>
            <div className="grid grid-cols-4 gap-3">
              {Object.entries(strategies).map(([key, label]) => (
                <button
                  key={key}
                  onClick={() => setStrategy(key)}
                  className={`px-4 py-3 rounded-lg font-medium transition-all ${
                    strategy === key
                      ? 'bg-orange-600 text-white shadow-lg'
                      : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                  }`}
                >
                  {label}
                </button>
              ))}
            </div>
          </div>

          <div className="grid grid-cols-2 gap-6">
            <div className="bg-gray-50 rounded-xl p-6">
              <h2 className="text-xl font-bold text-gray-800 mb-4 flex items-center gap-2">
                👥 配送员状态
              </h2>
              <div className="space-y-3">
                {drivers.map(driver => (
                  <div key={driver.id} className="bg-white rounded-lg p-4 shadow-sm card-hover">
                    <div className="flex items-center justify-between mb-3">
                      <div className="flex items-center gap-3">
                        <div className={`w-3 h-3 rounded-full ${
                          driver.status === 'busy' ? 'bg-red-500 pulse' : 'bg-green-500'
                        }`} />
                        <span className="font-semibold text-gray-800">{driver.name}</span>
                        <span className="text-2xl">{getVehicleEmoji(driver.vehicleType)}</span>
                      </div>
                      <span className="text-sm text-gray-600">
                        {driver.currentLoad}/{driver.capacity} 单
                      </span>
                    </div>
                    <div className="mb-2">
                      <div className="flex justify-between text-xs text-gray-600 mb-1">
                        <span>负载率</span>
                        <span>{Math.round((driver.currentLoad / driver.capacity) * 100)}%</span>
                      </div>
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div
                          className="bg-orange-600 h-2 rounded-full transition-all"
                          style={{ width: `${(driver.currentLoad / driver.capacity) * 100}%` }}
                        />
                      </div>
                    </div>
                    <div className="flex justify-between text-xs">
                      <span className="text-gray-600">{driver.vehicleType}</span>
                      <span className="text-yellow-600 font-medium">⭐ {driver.rating}</span>
                      <span className="text-gray-500">{driver.completedOrders} 单</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div className="bg-gray-50 rounded-xl p-6">
              <h2 className="text-xl font-bold text-gray-800 mb-4 flex items-center gap-2">
                📦 最近订单
              </h2>
              <div className="space-y-2 max-h-96 overflow-y-auto">
                {orders.length === 0 ? (
                  <p className="text-gray-500 text-center py-8">暂无订单，点击"开始"按钮生成订单</p>
                ) : (
                  orders.slice().reverse().map(order => (
                    <div key={order.id} className="bg-white rounded-lg p-4 shadow-sm card-hover fade-in">
                      <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center gap-2">
                          <span className="text-2xl">{order.icon}</span>
                          <span className="font-semibold text-gray-800">{order.type}</span>
                        </div>
                        <span className={`text-xs px-2 py-1 rounded-full ${getStatusColor(order.status)}`}>
                          {order.status === 'dispatched' ? '已派单' : '待派单'}
                        </span>
                      </div>
                      <div className="grid grid-cols-3 gap-2 text-xs text-gray-600 mb-2">
                        <div className="flex items-center gap-1">
                          📍 <span>{order.distance} km</span>
                        </div>
                        <div>💰 ¥{order.price}</div>
                        <div>⏱️ {order.estimatedTime} 分钟</div>
                      </div>
                      <div className="flex items-center justify-between text-sm">
                        <span className="text-gray-500">{order.createTime}</span>
                        {order.assignedTo && (
                          <span className="text-orange-600 font-medium">
                            → {order.assignedTo}
                          </span>
                        )}
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>
          </div>

          <div className="mt-6 bg-gradient-to-r from-orange-100 to-yellow-100 rounded-xl p-4">
            <div className="flex items-center gap-3">
              <span className="text-2xl">⚙️</span>
              <div>
                <p className="font-semibold text-gray-800">当前策略: {strategies[strategy]}</p>
                <p className="text-sm text-gray-600">
                  {strategy === 'nearest_first' && '优先将订单分配给距离最近的配送员'}
                  {strategy === 'load_balanced' && '平衡所有配送员的订单负载'}
                  {strategy === 'rating_priority' && '优先分配给高评分配送员'}
                  {strategy === 'capacity_match' && '根据订单类型匹配合适的运力'}
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

// 配置 API 地址
const API_BASE_URL = 'http://127.0.0.1:8080';

// API 调用函数
const api = {
    // 派单
    dispatch: async (orderData) => {
        const response = await fetch(`${API_BASE_URL}/api/dispatch`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(orderData)
        });
        return response.json();
    },

    // 获取配送员列表
    getDrivers: async () => {
        const response = await fetch(`${API_BASE_URL}/api/drivers`);
        return response.json();
    },

    // 添加配送员
    addDriver: async (driverData) => {
        const response = await fetch(`${API_BASE_URL}/api/drivers`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(driverData)
        });
        return response.json();
    },

    // 更新配送员位置
    updateLocation: async (driverId, location) => {
        const response = await fetch(`${API_BASE_URL}/api/drivers/location`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ driver_id: driverId, ...location })
        });
        return response.json();
    },

    // 释放订单
    releaseOrder: async (driverId) => {
        const response = await fetch(`${API_BASE_URL}/api/drivers/${driverId}/release`, {
            method: 'POST'
        });
        return response.json();
    },

    // 切换策略
    changeStrategy: async (strategy) => {
        const response = await fetch(`${API_BASE_URL}/api/strategy`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ strategy })
        });
        return response.json();
    },

    // 健康检查
    healthCheck: async () => {
        const response = await fetch(`${API_BASE_URL}/health`);
        return response.json();
    }
};

// 在组件中使用 API（示例）
// 将 dispatchOrder 函数修改为调用真实 API
const dispatchOrderWithAPI = async (order) => {
    try {
        const result = await api.dispatch({
            order_type: order.type,
            pickup_lat: order.pickupLocation.lat,
            pickup_lng: order.pickupLocation.lng,
            delivery_lat: order.deliveryLocation.lat,
            delivery_lng: order.deliveryLocation.lng
        });

        if (result.success) {
            console.log('派单成功:', result.data);
            return result.data;
        } else {
            console.error('派单失败:', result.message);
            return null;
        }
    } catch (error) {
        console.error('API 调用失败:', error);
        return null;
    }
};

// 渲染应用
ReactDOM.render(<DispatcherDemo />, document.getElementById('root'));