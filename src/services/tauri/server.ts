import type { ServerStatus, ModelTestResult, ModelMapping } from '../../types';
import { invokeTyped, COMMANDS } from './client';

export async function startServer(): Promise<ServerStatus> {
  return invokeTyped<ServerStatus>(COMMANDS.server.startServer);
}

export async function stopServer(): Promise<ServerStatus> {
  return invokeTyped<ServerStatus>(COMMANDS.server.stopServer);
}

export async function getServerStatus(): Promise<ServerStatus> {
  return invokeTyped<ServerStatus>(COMMANDS.server.getServerStatus);
}

export async function testModelConnection(localName: string): Promise<ModelTestResult> {
  return invokeTyped<ModelTestResult>(COMMANDS.server.testModelConnection, { localName } as Record<string, unknown>);
}

export async function testModelConfig(model: ModelMapping): Promise<ModelTestResult> {
  return invokeTyped<ModelTestResult>(COMMANDS.server.testModelConfig, { model } as Record<string, unknown>);
}
