<template>
  <div class="local-models-page">
    <div class="page-header">
      <div class="header-left">
        <button class="back-btn" @click="$emit('back')">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="19" y1="12" x2="5" y2="12"></line>
            <polyline points="12,19 5,12 12,5"></polyline>
          </svg>
        </button>
        <h2>本地模型</h2>
      </div>
    </div>

    <div class="settings-content">
      <div class="card-section">
        <div class="settings-inner-card">
          <div class="settings-inner-header">
            <h3>环境检测</h3>
            <button class="env-refresh-btn" @click="checkEnvironment" :disabled="!!installingPackage">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" :class="{ spinning: environmentStatus === null }">
                <polyline points="23,4 23,10 17,10"></polyline>
                <polyline points="1,20 1,14 7,14"></polyline>
                <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
              </svg>
              <span>重新检测</span>
            </button>
          </div>
            <div class="settings-inner-content">
              <div class="env-check-list" v-if="environmentStatus">
                <h4 class="env-group-title">系统状态</h4>
                <div class="env-check-row" :class="{ ok: environmentStatus.python, fail: !environmentStatus.python }">
                  <span class="env-check-name">
                    Python
                    <span class="env-check-ver" v-if="environmentStatus.python_version">{{ environmentStatus.python_version }}</span>
                  </span>
                  <span class="env-check-status">
                    <span class="env-check-ok" v-if="environmentStatus.python">正常</span>
                    <span class="env-check-missing" v-else>未安装</span>
                  </span>
                  <button class="env-info-btn" @click="showDepInfo('python')" title="查看详情">ⓘ</button>
                </div>
                <div class="env-check-row" :class="{ ok: discreteGpus.length > 0, fail: discreteGpus.length === 0 }">
                  <span class="env-check-name">GPU</span>
                  <span class="env-check-status">
                    <template v-if="environmentStatus.gpus.length > 0">
                      <span class="env-check-ok" v-for="(gpu, i) in environmentStatus.gpus" :key="i" :title="gpu.name + (gpu.gpu_type === 'integrated' ? ' (集成显卡)' : gpu.gpu_type === 'discrete' ? ' (独立显卡)' : '')">
                        {{ gpu.vendor }}{{ gpu.gpu_type === 'integrated' ? '(集)' : '' }}
                      </span>
                    </template>
                    <span class="env-check-missing" v-else>未检测到</span>
                  </span>
                  <button class="env-info-btn" @click="showDepInfo('gpu')" title="查看详情">ⓘ</button>
                </div>
                <div class="env-check-row" :class="{ ok: environmentStatus.modelscope, fail: !environmentStatus.modelscope }">
                  <span class="env-check-name">ModelScope</span>
                  <span class="env-check-status">
                    <span class="env-check-ok" v-if="environmentStatus.modelscope">正常</span>
                    <button v-else class="env-install-btn" @click="handleInstallDep('modelscope')" :disabled="!!installingPackage">
                      <svg v-if="installingPackage === 'modelscope'" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
                        <line x1="12" y1="2" x2="12" y2="6"></line><line x1="12" y1="18" x2="12" y2="22"></line>
                        <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line><line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
                      </svg>
                      {{ installBtnText('modelscope') }}
                    </button>
                  </span>
                  <button class="env-info-btn" @click="showDepInfo('modelscope')" title="查看详情">ⓘ</button>
                </div>

                <h4 class="env-group-title">
                  推理后端
                  <span class="env-group-hint" v-if="envRecommendation">{{ envRecommendation }}</span>
                </h4>
                <div class="env-check-row" :class="{ ok: environmentStatus.llama_cpp, fail: !environmentStatus.llama_cpp }">
                  <span class="env-check-name">llama.cpp</span>
                  <span class="env-check-status">
                    <span class="env-check-ok" v-if="environmentStatus.llama_cpp">已安装</span>
                    <span class="env-recommend-tag" v-if="!environmentStatus.llama_cpp && envRecommendLLamacpp">推荐</span>
                    <button v-if="!environmentStatus.llama_cpp" class="env-install-btn" @click="installDependency('llama-cpp-python')" :disabled="!!installingPackage">
                      <svg v-if="installingPackage === 'llama-cpp-python'" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
                        <line x1="12" y1="2" x2="12" y2="6"></line><line x1="12" y1="18" x2="12" y2="22"></line>
                        <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line><line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
                      </svg>
                      {{ installBtnText('llama-cpp-python') }}
                    </button>
                  </span>
                  <button class="env-info-btn" @click="showDepInfo('llamacpp')" title="查看详情">ⓘ</button>
                </div>
                <div class="env-check-row" :class="{ ok: environmentStatus.oneapi, fail: !environmentStatus.oneapi }" v-if="environmentStatus.gpus.some(g => g.vendor === 'Intel' && g.gpu_type === 'discrete')">
                  <span class="env-check-name">Intel oneAPI</span>
                  <span class="env-check-status">
                    <span class="env-check-ok" v-if="environmentStatus.oneapi">已安装</span>
                    <span class="env-recommend-tag" v-if="!environmentStatus.oneapi">推荐</span>
                    <button v-if="!environmentStatus.oneapi" class="env-install-btn" @click="installDependency('oneapi')">
                      <svg v-if="installingPackage === 'oneapi'" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
                        <line x1="12" y1="2" x2="12" y2="6"></line><line x1="12" y1="18" x2="12" y2="22"></line>
                        <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line><line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
                      </svg>
                      {{ installingPackage === 'oneapi' ? (dependencyInstallMessage || '打开中...') : '下载' }}
                    </button>
                  </span>
                  <button class="env-info-btn" @click="showDepInfo('oneapi')" title="查看详情">ⓘ</button>
                </div>
                <div class="env-check-row" :class="{ ok: environmentStatus.openvino, fail: !environmentStatus.openvino }">
                  <span class="env-check-name">
                    OpenVINO
                    <span class="env-check-ver" v-if="environmentStatus.openvino_version">{{ environmentStatus.openvino_version }}</span>
                  </span>
                  <span class="env-check-status">
                    <span class="env-check-ok" v-if="environmentStatus.openvino">正常</span>
                    <button v-else class="env-install-btn" @click="handleInstallDep('openvino')" :disabled="!!installingPackage">
                      <svg v-if="installingPackage === 'openvino'" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
                        <line x1="12" y1="2" x2="12" y2="6"></line><line x1="12" y1="18" x2="12" y2="22"></line>
                        <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line><line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
                      </svg>
                      {{ installBtnText('openvino') }}
                    </button>
                  </span>
                  <button class="env-info-btn" @click="showDepInfo('openvino')" title="查看详情">ⓘ</button>
                </div>
                <div class="env-check-row" :class="{ ok: environmentStatus.openvino_genai, fail: !environmentStatus.openvino_genai }">
                  <span class="env-check-name">OpenVINO GenAI</span>
                  <span class="env-check-status">
                    <span class="env-check-ok" v-if="environmentStatus.openvino_genai">正常</span>
                    <button v-else class="env-install-btn" @click="handleInstallDep('openvino-genai')" :disabled="!!installingPackage">
                      <svg v-if="installingPackage === 'openvino-genai'" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
                        <line x1="12" y1="2" x2="12" y2="6"></line><line x1="12" y1="18" x2="12" y2="22"></line>
                        <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line><line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
                      </svg>
                      {{ installBtnText('openvino-genai') }}
                    </button>
                  </span>
                  <button class="env-info-btn" @click="showDepInfo('openvino-genai')" title="查看详情">ⓘ</button>
                </div>
                <div class="env-check-row" :class="{ ok: environmentStatus.optimum, fail: !environmentStatus.optimum }">
                  <span class="env-check-name">Optimum (Intel)</span>
                  <span class="env-check-status">
                    <span class="env-check-ok" v-if="environmentStatus.optimum">正常</span>
                    <button v-else class="env-install-btn" @click="handleInstallDep('optimum[openvino]')" :disabled="!!installingPackage">
                      <svg v-if="installingPackage === 'optimum[openvino]'" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
                        <line x1="12" y1="2" x2="12" y2="6"></line><line x1="12" y1="18" x2="12" y2="22"></line>
                        <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line><line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
                      </svg>
                      {{ installBtnText('optimum[openvino]') }}
                    </button>
                  </span>
                  <button class="env-info-btn" @click="showDepInfo('optimum')" title="查看详情">ⓘ</button>
                </div>
                <div class="env-check-row" :class="{ ok: environmentStatus.transformers, fail: !environmentStatus.transformers }">
                  <span class="env-check-name">Transformers</span>
                  <span class="env-check-status">
                    <span class="env-check-ok" v-if="environmentStatus.transformers">已安装</span>
                    <button v-else class="env-install-btn" @click="handleInstallDep('transformers')" :disabled="!!installingPackage">
                      <svg v-if="installingPackage === 'transformers'" viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
                        <line x1="12" y1="2" x2="12" y2="6"></line><line x1="12" y1="18" x2="12" y2="22"></line>
                        <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line><line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
                      </svg>
                      {{ installBtnText('transformers') }}
                    </button>
                  </span>
                  <button class="env-info-btn" @click="showDepInfo('transformers')" title="查看详情">ⓘ</button>
                </div>
              </div>
              <div class="env-loading" v-else>
                <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
                  <line x1="12" y1="2" x2="12" y2="6"></line>
                  <line x1="12" y1="18" x2="12" y2="22"></line>
                  <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line>
                  <line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
                  <line x1="2" y1="12" x2="6" y2="12"></line>
                  <line x1="18" y1="12" x2="22" y2="12"></line>
                  <line x1="4.93" y1="19.07" x2="7.76" y2="16.24"></line>
                  <line x1="16.24" y1="7.76" x2="19.07" y2="4.93"></line>
                </svg>
                <span>正在检测环境...</span>
              </div>
              <div class="dep-install-msg" v-if="dependencyInstallMessage && !installingPackage">
                <span>{{ dependencyInstallMessage }}</span>
              </div>
            </div>
          </div>
        </div>

        <div class="card-section">
          <div class="settings-inner-card">
            <div class="settings-inner-header">
              <h3>模型存储</h3>
            </div>
            <div class="settings-inner-content">
              <div class="models-dir-row">
                <span class="models-dir-label">下载目录</span>
                <div class="models-dir-path">
                  <span class="models-dir-value">{{ modelsDir || defaultModelsDir }}</span>
                  <span class="models-dir-default" v-if="modelsDir">（自定义）</span>
                  <span class="models-dir-default" v-else>（默认）</span>
                </div>
                <div class="models-dir-actions">
                  <button class="models-dir-btn" @click="handleChangeModelsDir">更改</button>
                  <button class="models-dir-btn models-dir-reset" v-if="modelsDir" @click="setModelsDir(null)">重置</button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="card-section">
          <div class="settings-inner-card">
            <div class="settings-inner-header">
              <h3>安装模型</h3>
            </div>
            <div class="settings-inner-content">
              <div class="install-row">
                <input
                  class="search-input"
                  v-model="modelIdInput"
                  placeholder="输入模型 ID，例如 Qwen/Qwen3.5-0.8B"
                  @keyup.enter="handleInstallModel"
                />
                <button class="search-btn" @click="handleInstallModel" :disabled="isDownloading || !isValidModelId">
                  <svg v-if="!isDownloading" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                    <polyline points="7,10 12,15 17,10"></polyline>
                    <line x1="12" y1="15" x2="12" y2="3"></line>
                  </svg>
                  <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" class="spinning">
                    <line x1="12" y1="2" x2="12" y2="6"></line>
                    <line x1="12" y1="18" x2="12" y2="22"></line>
                    <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line>
                    <line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
                    <line x1="2" y1="12" x2="6" y2="12"></line>
                    <line x1="18" y1="12" x2="22" y2="12"></line>
                    <line x1="4.93" y1="19.07" x2="7.76" y2="16.24"></line>
                    <line x1="16.24" y1="7.76" x2="19.07" y2="4.93"></line>
                  </svg>
                  <span>{{ isDownloading ? '下载中...' : '下载' }}</span>
                </button>
              </div>

              <div class="download-progress-area" v-if="isDownloading || downloadProgress">
                <div class="download-info">
                  <span class="download-file" :class="{ 'download-error': downloadProgress?.status === 'error' }">
                    {{ downloadProgress?.status === 'error' ? '下载失败' : (downloadProgress?.status === 'cancelled' ? '已取消' : (downloadProgress?.message || '准备下载...')) }}
                  </span>
                  <span v-if="downloadProgress?.status !== 'error' && downloadProgress?.status !== 'cancelled'" class="download-percent">{{ downloadProgress?.progress_percent || 0 }}%</span>
                </div>
                <div class="progress-bar" v-if="downloadProgress?.status !== 'error' && downloadProgress?.status !== 'cancelled'">
                  <div class="progress-fill" :style="{ width: (downloadProgress?.progress_percent || 0) + '%' }"></div>
                </div>
                <div class="download-actions-row">
                  <button v-if="isDownloading" class="cancel-download-btn" @click="handleCancelDownload">取消</button>
                  <button v-if="!isDownloading && downloadProgress" class="cancel-download-btn" @click="downloadProgress = null">关闭</button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="card-section">
          <div class="settings-inner-card">
            <div class="settings-inner-header">
              <h3>本地模型</h3>
            </div>
            <div class="settings-inner-content">
              <div class="deploy-error-banner" v-if="deployError">
                <span class="deploy-error-text">{{ deployError }}</span>
                <button class="deploy-error-close" @click="deployError = null">关闭</button>
              </div>
              <div class="models-list" v-if="models.length > 0">
                <div v-for="model in models" :key="model.id" class="model-card">
                  <div class="model-left">
                    <div class="model-icon">{{ getTypeIcon(model.modelType) }}</div>
                  </div>
                  <div class="model-center">
                    <div class="model-display-name">{{ model.displayName }}</div>
                    <div class="model-id-text">{{ model.modelId }}</div>
                    <div class="model-meta-row">
                      <span class="type-badge" :style="{ background: getTypeColor(model.modelType) }">
                        {{ getTypeLabel(model.modelType) }}
                      </span>
                      <span class="model-size" v-if="model.sizeBytes">{{ formatSize(model.sizeBytes) }}</span>
                    </div>
                    <div class="model-path" v-if="model.localPath">{{ model.localPath }}</div>
                  </div>
                  <div class="model-right">
                    <div class="model-status" :class="model.status">
                      <span class="status-dot"></span>
                      <span class="status-text">{{ getStatusText(model.status) }}</span>
                    </div>
                    <div class="model-actions">
                      <button
                        v-if="model.status !== 'running'"
                        class="model-action-btn deploy"
                        @click="handleDeploy(model)"
                        :disabled="model.status === 'downloading' || model.status === 'converting' || model.status === 'stopping'"
                      >
                        部署
                      </button>
                      <button
                        v-if="model.status === 'running'"
                        class="model-action-btn stop"
                        @click="handleStop(model.id)"
                      >
                        停止
                      </button>
                      <button class="model-action-btn config" @click="openDeployConfig(model)">
                        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                          <circle cx="12" cy="12" r="3"></circle>
                          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
                        </svg>
                      </button>
                      <button
                        v-if="model.status === 'running'"
                        class="model-action-btn add-api"
                        @click="handleAddAsProvider(model)"
                      >
                        添加为API
                      </button>
                      <button class="model-action-btn delete" @click="openDeleteConfirm(model)">
                        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
                          <polyline points="3,6 5,6 21,6"></polyline>
                          <path d="M19,6v14a2,2,0,0,1-2,2H7a2,2,0,0,1-2-2V6m3,0V4a2,2,0,0,1,2-2h4a2,2,0,0,1,2,2v2"></path>
                        </svg>
                      </button>
                    </div>
                  </div>
                </div>
              </div>

              <div class="empty-state" v-else>
                <div class="empty-icon">
                  <svg viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
                    <rect x="2" y="2" width="20" height="20" rx="4" ry="4"></rect>
                    <path d="M7 8h10"></path>
                    <path d="M7 12h7"></path>
                    <path d="M7 16h4"></path>
                  </svg>
                </div>
                <p>还没有本地模型</p>
                <span>在上方输入模型 ID 下载安装</span>
              </div>
            </div>
          </div>
        </div>
      </div>

    <BaseDialog v-model="showDeployConfig" title="部署配置">
      <div class="form-group">
        <label>推理后端</label>
        <select class="device-select" v-model="deployConfigForm.backend">
          <option value="llama-cpp">llama.cpp (GGUF)</option>
          <option value="openvino">OpenVINO (IR)</option>
          <option value="transformers">Transformers (Safetensors)</option>
        </select>
        <span class="form-hint">GGUF 模型请选 llama.cpp，OpenVINO IR 模型请选 OpenVINO</span>
      </div>
      <div class="form-group">
        <label>端口</label>
        <input type="number" v-model.number="deployConfigForm.port" placeholder="8000" min="1024" max="65535" />
        <span class="form-hint">默认端口 8000</span>
      </div>
      <div class="form-group">
        <label>推理设备</label>
        <select class="device-select" v-model="deployConfigForm.device">
          <option value="GPU" v-if="discreteGpus.length > 0">GPU ({{ discreteGpuLabel }})</option>
          <option value="CPU">CPU</option>
        </select>
        <span class="form-hint" v-if="discreteGpus.length > 0">选择 GPU 可获得更快推理速度</span>
        <span class="form-hint" v-else>未检测到独立显卡，仅 CPU 可用</span>
      </div>
      <div class="form-group">
        <label>上下文长度</label>
        <input type="number" v-model.number="deployConfigForm.ctxSize" placeholder="2048" min="256" />
      </div>
      <div class="form-group">
        <label>线程数</label>
        <input type="number" v-model.number="deployConfigForm.threads" placeholder="4" min="1" />
      </div>
      <template #actions>
        <button class="cancel-btn" @click="showDeployConfig = false">取消</button>
        <button class="save-btn" @click="handleConfirmDeploy">确认部署</button>
      </template>
    </BaseDialog>

    <BaseDialog v-model="showDepInfoDialog" :title="depInfoData?.title || ''">
      <div class="dep-info-content">
        <p class="dep-info-desc">{{ depInfoData?.desc }}</p>
        <div class="dep-info-section" v-if="depInfoData?.pros">
          <span class="dep-info-label">✅ 优点</span>
          <ul><li v-for="(p, i) in depInfoData.pros" :key="i">{{ p }}</li></ul>
        </div>
        <div class="dep-info-section" v-if="depInfoData?.cons">
          <span class="dep-info-label">⚠️ 注意</span>
          <ul><li v-for="(c, i) in depInfoData.cons" :key="i">{{ c }}</li></ul>
        </div>
        <div class="dep-info-section" v-if="depInfoData?.useCases">
          <span class="dep-info-label">🎯 适用场景</span>
          <ul><li v-for="(u, i) in depInfoData.useCases" :key="i">{{ u }}</li></ul>
        </div>
      </div>
      <template #actions>
        <button class="save-btn" @click="showDepInfoDialog = false">知道了</button>
      </template>
    </BaseDialog>

    <BaseDialog v-model="showDeleteConfirm" title="确认删除">
      <div class="delete-warning">
        <svg viewBox="0 0 24 24" width="40" height="40" fill="none" stroke="#E74C3C" stroke-width="1.5">
          <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
          <line x1="12" y1="9" x2="12" y2="13"></line>
          <line x1="12" y1="17" x2="12.01" y2="17"></line>
        </svg>
        <p>确定要删除模型「<strong>{{ deletingModel?.displayName }}</strong>」吗？</p>
        <span>此操作不可恢复。</span>
      </div>
      <template #actions>
        <button class="cancel-btn" @click="showDeleteConfirm = false">取消</button>
        <button class="danger-btn" @click="handleConfirmDelete">确认删除</button>
      </template>
    </BaseDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import BaseDialog from './BaseDialog.vue'
import { useModelManager } from '../composables/useModelManager'
import type { LocalModel, InferenceBackend } from '../stores/localModels'
import { getDefaultDeployConfig, updateLocalModel, getBackendForFormat } from '../stores/localModels'
import { open } from '@tauri-apps/plugin-dialog'

defineEmits<{
  back: []
}>()

const {
  environmentStatus,
  isDownloading,
  downloadProgress,
  models,
  deployError,
  installingPackage,
  dependencyInstallMessage,
  checkEnvironment,
  downloadModel,
  cancelDownload,
  deployModel,
  stopModel,
  deleteModel,
  addAsProvider,
  installDependency,
  modelsDir,
  defaultModelsDir,
  setModelsDir,
} = useModelManager()

const modelIdInput = ref('')

const isValidModelId = computed(() => {
  const k = modelIdInput.value.trim()
  return k.includes('/') && k.split('/').length === 2 && k.split('/').every(p => p.length > 0)
})

const showDeployConfig = ref(false)
const showDeleteConfirm = ref(false)
const showDepInfoDialog = ref(false)
const depInfoData = ref<{ title: string; desc: string; pros?: string[]; cons?: string[]; useCases?: string[] } | null>(null)

interface DepInfoItem {
  title: string
  desc: string
  pros?: string[]
  cons?: string[]
  useCases?: string[]
}

const depInfoMap: Record<string, DepInfoItem> = {
  python: {
    title: 'Python',
    desc: 'Python 运行环境，所有推理后端和模型下载工具的基础依赖。',
    useCases: ['运行模型下载、转换、推理服务器等 Python 脚本', '必须安装 Python 3.9+，推荐 3.11 或 3.12'],
  },
  gpu: {
    title: 'GPU（图形处理器）',
    desc: '检测系统中的 GPU 设备。GPU 可以显著加速模型推理速度，通常比纯 CPU 快 5-20 倍。',
    pros: ['Intel Arc 系列 GPU 支持 SYCL 加速', 'NVIDIA GPU 支持 CUDA 加速', 'AMD GPU 支持 ROCm 加速'],
    cons: ['需要安装对应厂商的运行时库才能使用 GPU 加速'],
    useCases: ['Intel Arc + oneAPI → llama.cpp SYCL 后端', 'NVIDIA + CUDA → llama.cpp CUDA 后端', '无 GPU → 使用 CPU 推理，速度较慢'],
  },
  llamacpp: {
    title: 'llama.cpp',
    desc: '高性能 LLM 推理引擎，通过 GGUF 格式直接加载模型，无需预先转换。支持 CPU 和 GPU 加速。',
    pros: ['兼容性最好，几乎所有模型架构都支持', '无需预转换模型，直接加载 GGUF 文件', '支持 Intel GPU（SYCL）、NVIDIA GPU（CUDA）、CPU', '内存占用低，支持量化模型'],
    cons: ['纯 Python 绑定，GPU 加速需要对应版本的预编译包', '部分高级特性（如自定义采样器）不如专用推理引擎丰富'],
    useCases: ['✅ 推荐所有用户安装', '快速部署 GGUF 格式的本地模型', 'Intel Arc GPU 用户配合 oneAPI 可获得 GPU 加速'],
  },
  oneapi: {
    title: 'Intel oneAPI Base Toolkit',
    desc: 'Intel 提供的异构计算工具包，为 llama.cpp 提供 SYCL GPU 加速能力。仅在检测到 Intel GPU 时需要。',
    pros: ['免费使用', '让 Intel Arc GPU 可以加速 llama.cpp 推理', '提供 DPC++/C++ Compiler、oneMKL、TBB 等核心组件'],
    cons: ['安装体积较大（约 460MB）', '需要手动下载安装器并选择组件', '仅 Intel GPU 用户需要'],
    useCases: ['Intel Arc 系列 GPU 用户必装', '安装时只需勾选 DPC++/C++ Compiler、Math Kernel Library、Threading Building Blocks'],
  },
  openvino: {
    title: 'OpenVINO',
    desc: 'Intel 开源的 AI 推理优化工具包，支持将模型转换为 IR 格式后高效推理。',
    pros: ['Intel 官方优化，在 Intel 硬件上性能优异', '支持模型量化和压缩', '提供 OpenVINO GenAI 简化推理 API'],
    cons: ['需要将模型预转换为 IR 格式（.xml + .bin）', '部分新模型架构可能暂不支持转换', '仅支持 Intel 硬件优化'],
    useCases: ['Intel GPU/CPU 用户可选安装', '需要运行 OpenVINO IR 格式模型时安装'],
  },
  'openvino-genai': {
    title: 'OpenVINO GenAI',
    desc: '基于 OpenVINO 的生成式 AI 推理库，提供简化的 LLM 推理 API。',
    pros: ['API 简洁，一键加载和生成', '支持连续批处理等高级特性'],
    cons: ['依赖 OpenVINO 运行时', '模型仍需预转换为 IR 格式'],
    useCases: ['使用 OpenVINO 后端时的必需依赖'],
  },
  optimum: {
    title: 'Optimum (Intel)',
    desc: 'Hugging Face Optimum 的 Intel 后端扩展，提供将 HuggingFace 模型转换为 OpenVINO IR 格式的能力。',
    pros: ['无缝对接 HuggingFace 模型生态', '一行命令完成模型转换'],
    cons: ['依赖 OpenVINO 和 transformers', '部分新模型架构可能暂不支持'],
    useCases: ['需要将 safetensors 格式模型转换为 OpenVINO IR 格式时安装', '使用 OpenVINO 后端时的推荐依赖'],
  },
  modelscope: {
    title: 'ModelScope',
    desc: '阿里达摩院开源的模型社区平台，提供海量预训练模型的下载服务。支持从 ModelScope Hub 下载模型到本地。',
    pros: ['国内访问速度快，无需科学上网', '模型种类丰富，包含 LLM、多模态等', '支持断点续传和并行下载'],
    cons: ['部分模型可能不是最新版本', '与国际 HuggingFace 社区不完全同步'],
    useCases: ['✅ 推荐所有用户安装', '从 ModelScope Hub 搜索和下载模型'],
  },
  transformers: {
    title: 'Transformers',
    desc: 'Hugging Face 的核心库，支持直接加载 Safetensors 格式模型。兼容性最广，几乎所有模型架构都能直接运行，包括多模态模型。',
    pros: ['兼容性最广，支持几乎所有模型架构', '支持多模态模型（图片理解等）', '无需预转换模型格式', '使用模型自带的 chat template'],
    cons: ['需要 PyTorch 作为依赖（安装体积较大）', '推理速度通常比 llama.cpp 慢', '内存占用较高'],
    useCases: ['需要运行没有 GGUF 版本的模型时使用', '运行多模态模型（如 Gemma 3 视觉版）', '运行最新架构的模型'],
  },
}

function showDepInfo(key: string) {
  depInfoData.value = depInfoMap[key] || null
  showDepInfoDialog.value = true
}

const discreteGpus = computed(() => {
  if (!environmentStatus.value) return []
  return environmentStatus.value.gpus.filter((g: any) => g.gpu_type === 'discrete')
})

const discreteGpuLabel = computed(() => {
  const dgpus = discreteGpus.value
  if (dgpus.length === 0) return ''
  const vendors = [...new Set(dgpus.map((g: any) => g.vendor))]
  if (vendors.includes('NVIDIA')) return 'NVIDIA CUDA'
  if (vendors.includes('Intel')) return 'Intel Arc'
  if (vendors.includes('AMD')) return 'AMD'
  return vendors.join('/')
})

const envRecommendation = computed(() => {
  if (!environmentStatus.value) return ''
  const hasIntelDiscrete = environmentStatus.value.gpus.some((g: any) => g.vendor === 'Intel' && g.gpu_type === 'discrete')
  const hasNvidia = environmentStatus.value.gpus.some((g: any) => g.vendor === 'NVIDIA')
  const hasAmd = environmentStatus.value.gpus.some((g: any) => g.vendor === 'AMD')
  const hasDiscrete = discreteGpus.value.length > 0
  if (hasIntelDiscrete) return '💡 检测到 Intel Arc 独立显卡，推荐安装 llama.cpp + oneAPI 以获得 GPU 加速'
  if (hasNvidia) return '💡 检测到 NVIDIA 独立显卡，推荐安装 llama.cpp 以获得 CUDA 加速'
  if (hasAmd) return '💡 检测到 AMD 独立显卡，推荐安装 llama.cpp 以获得硬件加速'
  if (hasDiscrete) return '💡 检测到独立显卡，推荐安装 llama.cpp 以获得硬件加速'
  const hasIntegrated = environmentStatus.value.gpus.some((g: any) => g.gpu_type === 'integrated')
  if (hasIntegrated) return '💡 仅检测到集成显卡，推荐安装 llama.cpp 使用 CPU 推理'
  return '💡 纯 CPU 环境，推荐安装 llama.cpp 作为推理后端'
})

const envRecommendLLamacpp = computed(() => true)

function installBtnText(pkg: string): string {
  if (installingPackage.value !== pkg) return '安装'
  if (dependencyInstallMessage.value) {
    const msg = dependencyInstallMessage.value
    if (msg.length > 20) return msg.substring(0, 20) + '...'
    return msg
  }
  return '安装中...'
}
const editingModel = ref<LocalModel | null>(null)
const deletingModel = ref<LocalModel | null>(null)

const deployConfigForm = reactive({
  ctxSize: 2048,
  threads: 4,
  device: 'GPU',
  port: 8000,
  backend: 'llama-cpp' as InferenceBackend
})

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const k = 1024
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  const value = bytes / Math.pow(k, i)
  return value.toFixed(i === 0 ? 0 : 1) + ' ' + units[i]
}

function getTypeLabel(type: string): string {
  const map: Record<string, string> = {
    llm: '大语言模型',
    embedding: '嵌入模型',
    tts: '语音合成',
    other: '其他'
  }
  return map[type] || '其他'
}

function getTypeColor(type: string): string {
  const map: Record<string, string> = {
    llm: 'linear-gradient(135deg, #E67E22 0%, #F39C12 100%)',
    embedding: 'linear-gradient(135deg, #3498DB 0%, #5DADE2 100%)',
    tts: 'linear-gradient(135deg, #9B59B6 0%, #BB8FCE 100%)',
    other: 'linear-gradient(135deg, #95A5A6 0%, #BDC3C7 100%)'
  }
  return map[type] || map.other
}

function getTypeIcon(type: string): string {
  const map: Record<string, string> = {
    llm: '🤖',
    embedding: '📎',
    tts: '🔊',
    other: '📦'
  }
  return map[type] || '📦'
}

function getStatusText(status: string): string {
  const map: Record<string, string> = {
    running: '运行中',
    ready: '已停止',
    downloading: '下载中',
    converting: '转换中',
    error: '错误',
    stopping: '停止中'
  }
  return map[status] || status
}

async function handleInstallModel() {
  const id = modelIdInput.value.trim()
  if (!id || !isValidModelId.value) return
  const displayName = id.split('/').pop() || id
  downloadModel(id, displayName)
}

function handleInstallDep(packageName: string) {
  installDependency(packageName)
}

function handleCancelDownload() {
  if (downloadProgress.value) {
    cancelDownload(downloadProgress.value.model_id)
  }
}

async function handleChangeModelsDir() {
  const selected = await open({ directory: true, title: '选择模型存储目录' })
  if (selected && typeof selected === 'string') {
    await setModelsDir(selected)
  }
}

function openDeployConfig(model: LocalModel) {
  editingModel.value = model
  const config = model.deployConfig || getDefaultDeployConfig()
  deployConfigForm.ctxSize = config.ctxSize
  deployConfigForm.threads = config.threads
  deployConfigForm.device = config.device || (discreteGpus.value.length > 0 ? 'GPU' : 'CPU')
  deployConfigForm.port = config.port || 8000
  deployConfigForm.backend = config.backend || getBackendForFormat(model.modelFormat)
  showDeployConfig.value = true
}

function handleConfirmDeploy() {
  if (!editingModel.value) return
  const deployConfig = {
    port: deployConfigForm.port,
    ctxSize: deployConfigForm.ctxSize,
    threads: deployConfigForm.threads,
    device: deployConfigForm.device,
    backend: deployConfigForm.backend
  }
  updateLocalModel(editingModel.value.id, { deployConfig })
  const model = { ...editingModel.value, deployConfig }
  showDeployConfig.value = false
  deployModel(model)
}

function handleDeploy(model: LocalModel) {
  openDeployConfig(model)
}

function handleStop(modelId: string) {
  stopModel(modelId)
}

function openDeleteConfirm(model: LocalModel) {
  deletingModel.value = model
  showDeleteConfirm.value = true
}

function handleConfirmDelete() {
  if (!deletingModel.value) return
  deleteModel(deletingModel.value.id)
  showDeleteConfirm.value = false
  deletingModel.value = null
}

function handleAddAsProvider(model: LocalModel) {
  addAsProvider(model)
}
</script>

<style scoped>
.local-models-page {
  width: 100%;
  flex: 1;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, var(--color-surface) 0%, var(--color-surface-secondary) 50%, var(--color-surface-tertiary) 100%);
  overflow: hidden;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 28px;
  background: var(--color-surface-card);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.back-btn {
  width: 40px;
  height: 40px;
  border: none;
  background: var(--color-surface-secondary);
  cursor: pointer;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  flex-shrink: 0;
}

.back-btn:hover {
  background: var(--color-surface-tertiary);
  color: var(--color-text-primary);
}

.page-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  color: var(--color-text-primary);
  letter-spacing: -0.02em;
}

.settings-content {
  flex: 1;
  min-height: 0;
  padding: 20px 28px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  overflow-y: auto;
}

.card-section {
  width: 100%;
}

.settings-inner-card {
  border: 1px solid var(--color-border);
  border-radius: 14px;
  background: var(--color-surface);
  overflow: hidden;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.settings-inner-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface-card);
}

.settings-inner-header h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.settings-inner-content {
  padding: 20px;
}

.env-refresh-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 5px 12px;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: var(--color-surface-card);
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.env-refresh-btn:hover:not(:disabled) {
  color: var(--color-primary);
  border-color: var(--color-primary);
  background: var(--color-surface);
}

.env-refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.env-check-list {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.env-check-row {
  display: flex;
  align-items: center;
  padding: 10px 0;
  border-bottom: 1px solid var(--color-border);
  font-size: 13px;
}

.env-check-row:last-child {
  border-bottom: none;
}

.env-check-name {
  font-weight: 600;
  color: var(--color-text-primary);
  white-space: nowrap;
  flex-shrink: 0;
}

.env-check-status {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: auto;
  flex-shrink: 0;
}

.env-check-ok {
  color: var(--color-accent-success);
  font-weight: 600;
  font-size: 13px;
}

.env-check-ver {
  font-size: 11px;
  color: var(--color-text-muted);
}

.env-check-missing {
  color: var(--color-accent-error);
  font-weight: 500;
  font-size: 13px;
}

.env-check-row.ok .env-check-name {
  color: var(--color-text-primary);
}

.env-check-row.fail .env-check-name {
  color: var(--color-text-secondary);
}

.env-install-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 14px;
  border: none;
  border-radius: 8px;
  background: var(--color-primary-gradient);
  color: var(--color-text-inverse);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
  max-width: 240px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.env-install-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px var(--color-shadow-primary-strong);
}

.env-install-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.dep-install-msg {
  margin-top: 12px;
  padding: 10px 14px;
  background: var(--color-surface-card);
  border-radius: 8px;
  border: 1px solid var(--color-border);
  font-size: 12px;
  color: var(--color-text-secondary);
}

.models-dir-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 0;
}

.models-dir-label {
  font-size: 13px;
  color: var(--color-text-secondary);
  white-space: nowrap;
}

.models-dir-path {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 6px;
}

.models-dir-value {
  font-size: 13px;
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  direction: rtl;
  text-align: left;
}

.models-dir-default {
  font-size: 11px;
  color: var(--color-text-muted);
  white-space: nowrap;
}

.models-dir-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.models-dir-btn {
  padding: 4px 10px;
  font-size: 12px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-surface-card);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.15s ease;
}

.models-dir-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.models-dir-reset {
  color: var(--color-text-muted);
}

.models-dir-reset:hover {
  border-color: var(--color-error, #e53e3e);
  color: var(--color-error, #e53e3e);
}

.env-group-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 12px 0 6px 0;
  padding-bottom: 4px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  gap: 8px;
}

.env-group-title:first-child {
  margin-top: 0;
}

.env-group-hint {
  font-size: 11px;
  font-weight: 400;
  color: var(--color-primary);
  letter-spacing: 0;
  text-transform: none;
}

.env-recommend-tag {
  display: inline-block;
  font-size: 10px;
  padding: 1px 6px;
  background: var(--color-primary);
  color: white;
  border-radius: 3px;
  font-weight: 500;
}

.env-info-btn {
  background: none;
  border: none;
  color: var(--color-text-muted);
  cursor: pointer;
  font-size: 14px;
  padding: 0 4px;
  opacity: 0.6;
  transition: opacity 0.15s;
  flex-shrink: 0;
}

.env-info-btn:hover {
  opacity: 1;
  color: var(--color-primary);
}

.dep-info-content {
  padding: 4px 0;
}

.dep-info-desc {
  font-size: 13px;
  color: var(--color-text-secondary);
  line-height: 1.6;
  margin-bottom: 16px;
}

.dep-info-section {
  margin-bottom: 12px;
}

.dep-info-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text);
  display: block;
  margin-bottom: 4px;
}

.dep-info-section ul {
  margin: 0;
  padding-left: 18px;
}

.dep-info-section li {
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.8;
}

.env-loading {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 0;
  color: var(--color-text-muted);
  font-size: 13px;
}

.install-row {
  display: flex;
  gap: 10px;
  margin-bottom: 16px;
}

.search-input {
  flex: 1;
  padding: 12px 16px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  font-size: 14px;
  color: var(--color-text-primary);
  background: var(--color-surface-card);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.search-input:focus {
  outline: none;
  border-color: var(--color-primary);
  box-shadow: 0 0 0 3px var(--color-primary-alpha-15);
}

.search-input::placeholder {
  color: var(--color-text-muted);
}

.search-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 12px 20px;
  border: none;
  background: var(--color-primary-gradient);
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-inverse);
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 2px 8px var(--color-shadow-primary-hover);
  white-space: nowrap;
}

.search-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 16px var(--color-shadow-primary-strong);
}

.search-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.download-progress-area {
  padding: 16px;
  background: var(--color-surface-card);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  margin-bottom: 16px;
}

.download-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.download-file {
  font-size: 12px;
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 80%;
}

.download-percent {
  font-size: 13px;
  font-weight: 700;
  color: var(--color-primary);
  flex-shrink: 0;
}

.progress-bar {
  width: 100%;
  height: 8px;
  background: var(--color-border);
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 10px;
}

.progress-fill {
  height: 100%;
  background: var(--color-primary-gradient);
  border-radius: 4px;
  transition: width 0.3s ease;
}

.cancel-download-btn {
  padding: 6px 14px;
  border: 1px solid var(--color-accent-error);
  background: transparent;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-accent-error);
  cursor: pointer;
  transition: all 0.2s;
}

.cancel-download-btn:hover {
  background: var(--color-accent-error);
  color: var(--color-text-inverse);
}

.download-log {
  margin-bottom: 10px;
  padding: 8px 10px;
  background: var(--color-surface);
  border-radius: 6px;
  font-size: 11px;
  color: var(--color-text-secondary);
  font-family: 'Consolas', 'Monaco', monospace;
  word-break: break-all;
  max-height: 60px;
  overflow-y: auto;
}

.download-log::-webkit-scrollbar {
  width: 3px;
}

.download-log::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 2px;
}

.download-error {
  color: var(--color-accent-error) !important;
  font-weight: 600;
}

.download-actions-row {
  margin-top: 8px;
  display: flex;
  justify-content: flex-end;
}

.models-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.model-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 18px;
  border: 1px solid var(--color-border);
  border-radius: 12px;
  background: var(--color-surface-card);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.model-card:hover {
  border-color: var(--color-border-light);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px var(--color-shadow-alpha-08);
}

.model-left {
  flex-shrink: 0;
}

.model-icon {
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 22px;
  background: linear-gradient(135deg, var(--color-surface-secondary) 0%, var(--color-surface-tertiary) 100%);
  border-radius: 12px;
}

.model-center {
  flex: 1;
  min-width: 0;
}

.model-display-name {
  font-size: 14px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.model-id-text {
  font-size: 11px;
  color: var(--color-text-muted);
  margin-bottom: 6px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.model-meta-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.type-badge {
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 6px;
  color: var(--color-text-inverse);
  font-weight: 700;
  letter-spacing: 0.02em;
}

.model-size {
  font-size: 11px;
  color: var(--color-text-muted);
}

.model-path {
  font-size: 11px;
  color: var(--color-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.deploy-error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  background: var(--color-accent-error-alpha-10);
  border: 1px solid var(--color-accent-error-alpha-30);
  border-radius: 8px;
  margin-bottom: 12px;
}

.deploy-error-text {
  color: var(--color-accent-error);
  font-size: 13px;
  flex: 1;
  word-break: break-all;
}

.deploy-error-close {
  background: none;
  border: 1px solid var(--color-accent-error-alpha-30);
  color: var(--color-accent-error);
  padding: 2px 10px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  flex-shrink: 0;
  margin-left: 10px;
}

.deploy-error-close:hover {
  background: var(--color-accent-error-alpha-15);
}

.model-right {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 10px;
  flex-shrink: 0;
}

.model-status {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.model-status.ready .status-dot {
  background: var(--color-text-muted);
}

.model-status.running .status-dot {
  background: var(--color-accent-success);
  animation: statusPulse 2s ease-in-out infinite;
}

.model-status.downloading .status-dot {
  background: var(--color-accent-info);
  animation: statusPulse 1.5s ease-in-out infinite;
}

.model-status.error .status-dot {
  background: var(--color-accent-error);
}

.model-status.stopping .status-dot {
  background: var(--color-primary-light);
  animation: statusPulse 1s ease-in-out infinite;
}

@keyframes statusPulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.status-text {
  font-size: 12px;
  color: var(--color-text-secondary);
  font-weight: 500;
}

.model-actions {
  display: flex;
  gap: 6px;
}

.model-action-btn {
  padding: 7px 12px;
  border: 1px solid var(--color-border);
  background: var(--color-surface-card);
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 600;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  white-space: nowrap;
}

.model-action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.model-action-btn.deploy {
  border-color: var(--color-accent-success);
  color: var(--color-accent-success);
  background: rgba(39, 174, 96, 0.06);
}

.model-action-btn.deploy:hover:not(:disabled) {
  background: var(--color-accent-success);
  color: var(--color-text-inverse);
}

.model-action-btn.stop {
  border-color: var(--color-accent-error);
  color: var(--color-accent-error);
  background: var(--color-accent-error-alpha-06);
}

.model-action-btn.stop:hover:not(:disabled) {
  background: var(--color-accent-error);
  color: var(--color-text-inverse);
}

.model-action-btn.config {
  background: var(--color-surface-secondary);
  color: var(--color-text-secondary);
}

.model-action-btn.config:hover {
  background: var(--color-primary);
  color: var(--color-text-inverse);
  border-color: var(--color-primary);
}

.model-action-btn.add-api {
  border-color: var(--color-accent-info);
  color: var(--color-accent-info);
  background: rgba(52, 152, 219, 0.06);
}

.model-action-btn.add-api:hover:not(:disabled) {
  background: var(--color-accent-info);
  color: var(--color-text-inverse);
}

.model-action-btn.delete {
  background: var(--color-danger-bg);
  color: var(--color-text-secondary);
}

.model-action-btn.delete:hover {
  border-color: var(--color-accent-error);
  background: var(--color-accent-error);
  color: var(--color-text-inverse);
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.empty-state {
  text-align: center;
  padding: 50px 24px;
  color: var(--color-text-muted);
}

.empty-icon {
  margin-bottom: 16px;
  color: var(--color-border);
}

.empty-icon svg {
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { transform: scale(1); opacity: 0.6; }
  50% { transform: scale(1.05); opacity: 1; }
}

.empty-state p {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
}

.empty-state span {
  font-size: 13px;
}

.form-group {
  margin-bottom: 22px;
}

.form-group:last-child {
  margin-bottom: 0;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-size: 13px;
  font-weight: 700;
  color: var(--color-text-primary);
}

.form-group input {
  width: 100%;
  padding: 12px 16px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  font-size: 14px;
  color: var(--color-text-primary);
  background: var(--color-surface);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.form-group input:focus {
  outline: none;
  border-color: var(--color-primary);
  background: var(--color-surface-card);
  box-shadow: 0 0 0 3px var(--color-primary-alpha-15);
}

.form-group input::placeholder {
  color: var(--color-text-muted);
}

.form-group input:disabled {
  opacity: 0.6;
  background: var(--color-border);
  cursor: not-allowed;
}

.device-select {
  width: 100%;
  padding: 12px 16px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  font-size: 14px;
  color: var(--color-text-primary);
  background: var(--color-surface);
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  appearance: none;
  cursor: pointer;
}

.device-select:focus {
  outline: none;
  border-color: var(--color-primary);
  background: var(--color-surface-card);
  box-shadow: 0 0 0 3px var(--color-primary-alpha-15);
}

.form-hint {
  display: block;
  margin-top: 6px;
  font-size: 11px;
  color: var(--color-text-muted);
}

.cancel-btn {
  flex: 1;
  padding: 14px;
  border: 1px solid var(--color-border);
  background: var(--color-surface-card);
  border-radius: 10px;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.cancel-btn:hover {
  border-color: var(--color-text-muted);
  background: var(--color-surface-secondary);
}

.save-btn {
  flex: 2;
  padding: 14px;
  border: none;
  border-radius: 10px;
  background: var(--color-primary-gradient);
  color: var(--color-text-inverse);
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 4px 16px var(--color-shadow-primary-hover);
}

.save-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px var(--color-shadow-primary-strong);
}

.save-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.danger-btn {
  flex: 2;
  padding: 14px;
  border: none;
  border-radius: 10px;
  background: linear-gradient(135deg, var(--color-accent-error) 0%, var(--color-accent-dark-red) 100%);
  color: var(--color-text-inverse);
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 4px 16px var(--color-shadow-error);
}

.danger-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px var(--color-shadow-error-hover);
}

.delete-warning {
  text-align: center;
  padding: 10px 0;
}

.delete-warning svg {
  margin-bottom: 16px;
}

.delete-warning p {
  font-size: 14px;
  color: var(--color-text-primary);
  margin-bottom: 8px;
  line-height: 1.5;
}

.delete-warning span {
  font-size: 13px;
  color: var(--color-text-muted);
}
</style>
