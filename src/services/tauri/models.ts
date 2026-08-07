import { invoke } from '@tauri-apps/api/core';
import type { ModelGroup, ModelDefinition } from '../../types';

export async function getGroups(): Promise<ModelGroup[]> {
  return invoke<ModelGroup[]>('get_groups');
}

export async function saveGroup(group: ModelGroup): Promise<void> {
  return invoke('save_group', { group });
}

export async function deleteGroup(name: string): Promise<void> {
  return invoke('delete_group', { name });
}

export async function getModelDefinitions(): Promise<ModelDefinition[]> {
  return invoke<ModelDefinition[]>('get_model_definitions');
}

export async function saveModelDefinition(def: ModelDefinition): Promise<void> {
  return invoke('save_model_definition', { def });
}

export async function deleteModelDefinition(id: string): Promise<void> {
  return invoke('delete_model_definition', { id });
}
