/**
 * @license
 * SPDX-License-Identifier: Apache-2.0
 */

export interface UserProfile {
  name: string;
  job: string;
  location: string;
  mission: string;
  interests: string[];
  preferences: Record<string, string>;
  role?: string;
  alias?: string;
  age?: number;
  birth_date?: string;
  astrological_profile?: {
    sun_sign: string;
    moon_sign: string;
    planetary_influences: Record<string, string>;
  };
  numerology?: {
    life_path: string;
    details: string;
  };
  tarot_archetypes?: {
    birth_card: string;
    year_card_2025: string;
    soul_card: string;
  };
  mbti?: string;
  philosophical_core?: string;
}

export interface CoreMemory {
  user_profile: UserProfile;
  current_context: string;
}

export interface EpisodicMemory {
  id: string;
  timestamp: string;
  description: string;
  importance: 'low' | 'medium' | 'high';
}

export interface KnowledgeNode {
  id: string;
  label: string;
  type: 'person' | 'place' | 'object' | 'system' | 'concept';
}

export interface KnowledgeRelation {
  source: string;
  target: string;
  label: string;
}

export interface MemoryState {
  core: CoreMemory;
  episodic: EpisodicMemory[];
  relations: KnowledgeRelation[];
  nodes: KnowledgeNode[];
}

export interface TerminalLog {
  id: string;
  timestamp: string;
  command: string;
  output: string[];
  status: 'success' | 'running' | 'error';
}

export interface SystemStatus {
  cpu: number;
  ram: number;
  activeConnections: number;
  storage: string;
  status: string;
}

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  toolDetails?: {
    name: string;
    args: Record<string, any>;
    result?: string;
  };
}

export interface JarvisSkill {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
  category: 'core' | 'automation' | 'utility' | 'integration' | 'macro';
  icon?: string;
  scriptTrigger?: string;
  commands?: string[];
}

