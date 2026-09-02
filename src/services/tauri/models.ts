import type { ModelGroup, ModelDefinition } from '../../types';
import { invokeTyped, COMMANDS } from './client';

export async function getGroups(): Promise<ModelGroup[]> {
  return invokeTyped<ModelGroup[]>(COMMANDS.models.getGroups);
}

export async function saveGroup(group: ModelGroup): Promise<void> {
  return invokeTyped<void>(COMMANDS.models.saveGroup, { group } as Record<string, unknown>);
}

export async function deleteGroup(name: string): Promise<void> {
  return invokeTyped<void>(COMMANDS.models.deleteGroup, { name } as Record<string, unknown>);
}

export async function getModelDefinitions(): Promise<ModelDefinition[]> {
  return invokeTyped<ModelDefinition[]>(COMMANDS.models.getModelDefinitions);
}

export async function saveModelDefinition(def: ModelDefinition): Promise<void> {
  return invokeTyped<void>(COMMANDS.models.saveModelDefinition, { def } as Record<string, unknown>);
}

export async function deleteModelDefinition(id: string): Promise<void> {
  return invokeTyped<void>(COMMANDS.models.deleteModelDefinition, { id } as Record<string, unknown>);
}
