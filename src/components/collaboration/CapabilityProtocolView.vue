<script setup lang="ts">
import { ref, computed } from 'vue'
import type { AgentCapability, CollaborationProtocol } from '@/lib/collaboration/types'
import { useI18n } from '@/locales'

const { t } = useI18n()

// Props
interface Props {
  agentId: string
  agentName: string
  capabilities: AgentCapability[]
  protocols?: CollaborationProtocol[]
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  capabilityClick: [capabilityId: string]
  protocolClick: [protocolId: string]
}>()

// State
const activeTab = ref<'capabilities' | 'protocols'>('capabilities')
const selectedCapabilityId = ref<string | null>(null)
const selectedProtocolId = ref<string | null>(null)

// Computed
const capabilityCategories = computed(() => {
  const categories: Record<string, AgentCapability[]> = {}
  props.capabilities.forEach(cap => {
    if (!categories[cap.category]) {
      categories[cap.category] = []
    }
    categories[cap.category].push(cap)
  })
  return categories
})

const categoryIcons: Record<string, string> = {
  planning: '📋',
  execution: '⚡',
  review: '👀',
  testing: '🧪',
  communication: '💬',
  orchestration: '🎭',
}

const categoryNames: Record<string, string> = {
  planning: 'Planning',
  execution: 'Execution',
  review: 'Review',
  testing: 'Testing',
  communication: 'Communication',
  orchestration: 'Orchestration',
}

// Helper functions
function getProficiencyColor(proficiency: number): string {
  if (proficiency >= 80) return '#10B981'
  if (proficiency >= 60) return '#3B82F6'
  if (proficiency >= 40) return '#F59E0B'
  return '#EF4444'
}

function getProficiencyText(proficiency: number): string {
  if (proficiency >= 80) return t('capabilityProtocol.proficiencyExpert')
  if (proficiency >= 60) return t('capabilityProtocol.proficiencyAdvanced')
  if (proficiency >= 40) return t('capabilityProtocol.proficiencyIntermediate')
  return t('capabilityProtocol.proficiencyBasic')
}
</script>

<template>
  <div class="capability-protocol-view">
    <!-- Header -->
    <div class="view-header">
      <div class="header-title">
        <span class="agent-icon">🤖</span>
        <span class="agent-name">{{ agentName }}</span>
      </div>
      <div class="header-subtitle">{{ t('capabilityProtocol.title') }}</div>
    </div>

    <!-- Tab selector -->
    <div class="tab-selector">
      <button
        :class="['tab-button', { active: activeTab === 'capabilities' }]"
        @click="activeTab = 'capabilities'"
      >
        <span class="tab-icon">🎯</span>
        <span class="tab-label">{{ t('capabilityProtocol.capabilitiesTab') }}</span>
        <span class="tab-count">{{ capabilities.length }}</span>
      </button>
      <button
        v-if="protocols && protocols.length > 0"
        :class="['tab-button', { active: activeTab === 'protocols' }]"
        @click="activeTab = 'protocols'"
      >
        <span class="tab-icon">📜</span>
        <span class="tab-label">{{ t('capabilityProtocol.protocolsTab') }}</span>
        <span class="tab-count">{{ protocols.length }}</span>
      </button>
    </div>

    <!-- Capabilities tab -->
    <div v-if="activeTab === 'capabilities'" class="capabilities-content">
      <!-- Category sections -->
      <div
        v-for="(categoryCaps, category) in capabilityCategories"
        :key="category"
        class="capability-category"
      >
        <div class="category-header">
          <span class="category-icon">{{ categoryIcons[category] || '📦' }}</span>
          <span class="category-name">{{ categoryNames[category] || category }}</span>
          <span class="category-count">{{ categoryCaps.length }}</span>
        </div>

        <div class="capabilities-grid">
          <div
            v-for="cap in categoryCaps"
            :key="cap.id"
            class="capability-card"
            :class="{ selected: selectedCapabilityId === cap.id }"
            @click="selectedCapabilityId = cap.id; emit('capabilityClick', cap.id)"
          >
            <!-- Capability header -->
            <div class="cap-header">
              <span class="cap-icon">{{ cap.icon }}</span>
              <span class="cap-name">{{ cap.name }}</span>
            </div>

            <!-- Capability description -->
            <div class="cap-description">{{ cap.description }}</div>

            <!-- Proficiency indicator -->
            <div v-if="cap.proficiency" class="cap-proficiency">
              <div class="proficiency-bar">
                <div
                  class="proficiency-fill"
                  :style="{
                    width: `${cap.proficiency}%`,
                    backgroundColor: getProficiencyColor(cap.proficiency),
                  }"
                />
              </div>
              <span class="proficiency-text">{{ getProficiencyText(cap.proficiency) }}</span>
            </div>

            <!-- Prerequisites -->
            <div v-if="cap.prerequisites && cap.prerequisites.length > 0" class="cap-prerequisites">
              <span class="prereq-label">{{ t('capabilityProtocol.requires') }}:</span>
              <div class="prereq-list">
                <span v-for="prereq in cap.prerequisites.slice(0, 2)" :key="prereq" class="prereq-item">
                  {{ prereq }}
                </span>
              </div>
            </div>

            <!-- Outputs -->
            <div v-if="cap.outputs && cap.outputs.length > 0" class="cap-outputs">
              <span class="output-label">{{ t('capabilityProtocol.produces') }}:</span>
              <div class="output-list">
                <span v-for="output in cap.outputs.slice(0, 2)" :key="output" class="output-item">
                  {{ output }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Protocols tab -->
    <div v-if="activeTab === 'protocols' && protocols" class="protocols-content">
      <div
        v-for="protocol in protocols"
        :key="protocol.id"
        class="protocol-card"
        :class="{ selected: selectedProtocolId === protocol.id }"
        @click="selectedProtocolId = protocol.id; emit('protocolClick', protocol.id)"
      >
        <!-- Protocol header -->
        <div class="protocol-header">
          <div class="protocol-title">
            <span class="protocol-name">{{ protocol.name }}</span>
            <span class="protocol-version">v{{ protocol.version }}</span>
          </div>
          <div class="protocol-badge">
            {{ protocol.participants.length }} {{ t('capabilityProtocol.agents') }}
          </div>
        </div>

        <!-- Contract visualization -->
        <div class="protocol-contracts">
          <!-- Input contract -->
          <div class="contract-section input-contract">
            <div class="contract-label">
              <span class="contract-icon">📥</span>
              <span>{{ t('capabilityProtocol.input') }}</span>
            </div>
            <div class="contract-details">
              <div class="contract-type">{{ protocol.inputContract.type }}</div>
              <div v-if="protocol.inputContract.description" class="contract-desc">
                {{ protocol.inputContract.description }}
              </div>
            </div>
          </div>

          <!-- Output contract -->
          <div class="contract-section output-contract">
            <div class="contract-label">
              <span class="contract-icon">📤</span>
              <span>{{ t('capabilityProtocol.output') }}</span>
            </div>
            <div class="contract-details">
              <div class="contract-type">{{ protocol.outputContract.type }}</div>
              <div v-if="protocol.outputContract.description" class="contract-desc">
                {{ protocol.outputContract.description }}
              </div>
            </div>
          </div>
        </div>

        <!-- Participants -->
        <div class="protocol-participants">
          <span class="participants-label">{{ t('capabilityProtocol.participants') }}:</span>
          <div class="participants-list">
            <span v-for="participantId in protocol.participants.slice(0, 3)" :key="participantId" class="participant-badge">
              🤖 {{ participantId }}
            </span>
            <span v-if="protocol.participants.length > 3" class="participant-more">
              +{{ protocol.participants.length - 3 }} {{ t('capabilityProtocol.more') }}
            </span>
          </div>
        </div>

        <!-- Execution conditions -->
        <div v-if="protocol.executionConditions.length > 0" class="protocol-conditions">
          <span class="conditions-label">{{ t('capabilityProtocol.conditions') }}:</span>
          <div class="conditions-list">
            <div v-for="condition in protocol.executionConditions.slice(0, 2)" :key="condition.id" class="condition-item">
              <span class="condition-type">{{ condition.type }}</span>
              <span class="condition-desc">{{ condition.description }}</span>
            </div>
          </div>
        </div>

        <!-- Constraints -->
        <div v-if="protocol.constraints.length > 0" class="protocol-constraints">
          <span class="constraints-label">{{ t('capabilityProtocol.constraints') }}:</span>
          <div class="constraints-list">
            <div v-for="constraint in protocol.constraints.slice(0, 2)" :key="constraint.id" class="constraint-item">
              <span class="constraint-type">{{ constraint.type }}</span>
              <span class="constraint-value">{{ constraint.value }}{{ constraint.unit }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.capability-protocol-view {
  width: 100%;
  height: 100%;
  background: white;
  border-radius: 8px;
  overflow: hidden;
}

.view-header {
  padding: 16px;
  border-bottom: 1px solid #E5E7EB;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.agent-icon {
  font-size: 24px;
}

.agent-name {
  font-size: 16px;
  font-weight: 600;
  color: #1F2937;
}

.header-subtitle {
  font-size: 13px;
  color: #6B7280;
}

.tab-selector {
  display: flex;
  gap: 8px;
  padding: 8px 16px;
  background: #F9FAFB;
  border-bottom: 1px solid #E5E7EB;
}

.tab-button {
  flex: 1;
  padding: 8px 12px;
  background: white;
  border: 1px solid #E5E7EB;
  border-radius: 6px;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tab-button:hover {
  background: #F9FAFB;
}

.tab-button.active {
  background: #3B82F6;
  border-color: #3B82F6;
  color: white;
}

.tab-icon {
  font-size: 16px;
}

.tab-label {
  font-size: 13px;
  font-weight: 600;
}

.tab-count {
  padding: 2px 8px;
  background: rgba(0, 0, 0, 0.1);
  border-radius: 12px;
  font-size: 11px;
}

.tab-button.active .tab-count {
  background: rgba(255, 255, 255, 0.3);
}

.capabilities-content,
.protocols-content {
  padding: 16px;
  overflow-y: auto;
  max-height: 600px;
}

.capability-category {
  margin-bottom: 20px;
}

.category-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  padding: 8px 12px;
  background: #F3F4F6;
  border-radius: 6px;
}

.category-icon {
  font-size: 20px;
}

.category-name {
  font-size: 14px;
  font-weight: 600;
  color: #1F2937;
}

.category-count {
  padding: 2px 8px;
  background: white;
  border-radius: 12px;
  font-size: 11px;
  color: #6B7280;
}

.capabilities-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 12px;
}

.capability-card {
  background: white;
  border: 1px solid #E5E7EB;
  border-radius: 8px;
  padding: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.capability-card:hover {
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
  transform: translateY(-2px);
}

.capability-card.selected {
  border-color: #3B82F6;
  background: #EBF5FF;
}

.cap-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.cap-icon {
  font-size: 20px;
}

.cap-name {
  font-size: 14px;
  font-weight: 600;
  color: #1F2937;
}

.cap-description {
  font-size: 12px;
  color: #6B7280;
  line-height: 1.4;
  margin-bottom: 8px;
}

.cap-proficiency {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.proficiency-bar {
  flex: 1;
  height: 4px;
  background: #E5E7EB;
  border-radius: 2px;
  overflow: hidden;
}

.proficiency-fill {
  height: 100%;
  transition: width 0.3s ease;
}

.proficiency-text {
  font-size: 11px;
  color: #6B7280;
  font-weight: 600;
}

.cap-prerequisites,
.cap-outputs {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid #F3F4F6;
}

.prereq-label,
.output-label {
  font-size: 11px;
  color: #9CA3AF;
}

.prereq-list,
.output-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.prereq-item,
.output-item {
  padding: 2px 6px;
  background: #F3F4F6;
  border-radius: 4px;
  font-size: 10px;
  color: #4B5563;
}

.output-item {
  background: #EBF5FF;
  color: #3B82F6;
}

.protocols-content .protocol-card {
  background: white;
  border: 1px solid #E5E7EB;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.protocol-card:hover {
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
}

.protocol-card.selected {
  border-color: #3B82F6;
  background: #EBF5FF;
}

.protocol-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.protocol-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.protocol-name {
  font-size: 16px;
  font-weight: 600;
  color: #1F2937;
}

.protocol-version {
  padding: 2px 8px;
  background: #F3F4F6;
  border-radius: 4px;
  font-size: 11px;
  color: #6B7280;
}

.protocol-badge {
  padding: 4px 12px;
  background: #EBF5FF;
  border-radius: 12px;
  font-size: 12px;
  color: #3B82F6;
}

.protocol-contracts {
  display: flex;
  gap: 12px;
  margin-bottom: 12px;
}

.contract-section {
  flex: 1;
  padding: 12px;
  border-radius: 6px;
  border: 1px solid #E5E7EB;
}

.input-contract {
  background: #FEF3C7;
}

.output-contract {
  background: #D1FAE5;
}

.contract-label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  font-weight: 600;
  margin-bottom: 4px;
}

.contract-icon {
  font-size: 14px;
}

.contract-type {
  font-size: 11px;
  color: #4B5563;
  padding: 2px 6px;
  background: white;
  border-radius: 4px;
}

.contract-desc {
  font-size: 11px;
  color: #6B7280;
  margin-top: 4px;
}

.protocol-participants,
.protocol-conditions,
.protocol-constraints {
  margin-top: 8px;
}

.participants-label,
.conditions-label,
.constraints-label {
  font-size: 12px;
  color: #9CA3AF;
  margin-bottom: 4px;
}

.participants-list,
.conditions-list,
.constraints-list {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.participant-badge {
  padding: 4px 8px;
  background: #F3F4F6;
  border-radius: 4px;
  font-size: 11px;
  color: #4B5563;
}

.participant-more {
  padding: 4px 8px;
  background: #E5E7EB;
  border-radius: 4px;
  font-size: 11px;
  color: #6B7280;
}

.condition-item,
.constraint-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px;
  background: #F9FAFB;
  border-radius: 4px;
}

.condition-type,
.constraint-type {
  font-size: 10px;
  color: #3B82F6;
  font-weight: 600;
}

.condition-desc {
  font-size: 11px;
  color: #6B7280;
}

.constraint-value {
  font-size: 11px;
  color: #4B5563;
  font-weight: 600;
}
</style>