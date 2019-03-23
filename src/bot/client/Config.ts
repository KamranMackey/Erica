/**
 * Config.ts -- Ellie's configuration system.
 *
 * Copyright (c) 2018-present Kamran Mackey.
 *
 * Erica is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Erica is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Erica. If not, see <https://www.gnu.org/licenses/>.
 */

import * as fs from 'fs';
import * as path from 'path';
import * as toml from 'toml';

const configFile = path.join(__dirname, '..', '..', '..', 'config.toml');

type Configuration = {
  owner?: string;
  prefix?: string;
  token?: string;
  lastfm?: string;
  spotify?: {
    clientID?: string,
    clientSecret?: string;
  };
  google?: string;
  github?: {
    token?: string, // The personal authentication token.
    repo?: string; // The GitHub repository.
  };
  darksky?: {
    key?: string;
  }
  nexusmods?: string;
  repo?: string;
};

export default class Config {
  /** The command prefix used for invoking commands. */
  readonly prefix: string;
  /** The bot's owner. Used for commands that should be owner-only. */
  readonly owner: string;
  /** The Discord API token to use, used for connecting to the Discord API. */
  readonly token: string;
  /** The LastFM API key to use. */
  readonly lastfm: string;
  /** The Spotify client ID and client secret to use. */
  readonly spotify: { clientID: string, clientSecret: string };
  /** The Google API key to use. */
  readonly google: string;
  /** The GitHub authentication token and repository. */
  readonly github: { token: string, repo: string };
  /** The Dark Sky API key to use. */
  readonly darksky: { key: string };
  /** The Nexus Mods API key to use. */
  readonly nexusmods: string;
  /** The GitHub repository hosting the bot. */
  readonly repo: string;

  public constructor(string?: string) {
    const config = string ? (toml.parse(string) as Configuration) : {};
    const spotify = config.spotify || {};
    const darksky = config.darksky || {};
    const github = config.github || {};
    this.owner = config.owner || '';
    this.prefix = config.prefix || '!';
    this.token = config.token || '';
    this.lastfm = config.lastfm || '';
    this.spotify = {
      clientID: spotify.clientID || '',
      clientSecret: spotify.clientSecret || '',
    };
    this.github = {
      token: github.token || '',
      repo: github.repo || '',
    };
    this.darksky = { key: darksky.key || '' };
    this.nexusmods = config.nexusmods || '';
    this.google = config.google || '';
    this.repo = config.repo || '';
  }

  /** Loads the bot's configuration from a configuration file. */
  static loadConfig(path: string = configFile): Config {
    return new Config(fs.readFileSync(path, 'utf8'));
  }
}
