import { invoke } from '@tauri-apps/api/core';
import type { ServerStatus, ModelTestResult, ModelMapping } from '../../types';

export async function startServer(): Promise<ServerStatus> {
  return invoke<ServerStatus>('start_server');
}

export async function stopServer(): Promise<ServerStatus> {
  return invoke<ServerStatus>('stop_server');
}

export async function getServerStatus(): Promise<ServerStatus> {
  return invoke<ServerStatus>('get_server_status');
}

export async function testModelConnection(localName: string): Promise<ModelTestResult> {
  return invoke<ModelTestResult>('test_model_connection', { localName });
}

export async function testModelConfig(model: ModelMapping): Promise<ModelTestResult> {
  return invoke<ModelTestResult>('test_model_config', { model });
}
