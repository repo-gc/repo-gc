// Import-diversity: imports from many different packages
import express from 'express';
import { z } from 'zod';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import http from 'node:http';
import os from 'node:os';
import util from 'node:util';
import assert from 'node:assert';
import events from 'node:events';
import stream from 'node:stream';
import url from 'node:url';

export function diverseFunction(): void {
  const app = express();
  const schema = z.string();
  const home = os.homedir();
  const hash = crypto.createHash('sha256');
  const server = http.createServer();
  void app;
  void schema;
  void home;
  void hash;
  void server;
}
