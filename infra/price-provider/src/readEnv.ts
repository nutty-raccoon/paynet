import type { Token } from ".";

export type Env = {
  tokens: Record<string, Token[]>;
  currencies: string[];
  isPro: boolean;
  apiKey: string;
  host: string;
  port: number;
};

function assert(condition: any, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

export async function readEnv(): Promise<Env> {
  const {
    COIN_PRO_GECKO_API_KEY,
    COIN_DEMO_GECKO_API_KEY,
    TOKENS,
    CURRENCIES,
    HOST,
    PORT,
  } = process.env;

  assert(
    COIN_PRO_GECKO_API_KEY || COIN_DEMO_GECKO_API_KEY,
    'Missing env var: either COIN_PRO_GECKO_API_KEY or COIN_DEMO_GECKO_API_KEY must be set',
  );
  const isPro = Boolean(COIN_PRO_GECKO_API_KEY);
  const apiKey = isPro ? COIN_PRO_GECKO_API_KEY! : COIN_DEMO_GECKO_API_KEY!;

  assert(TOKENS, 'Missing env var: TOKENS');
  assert(CURRENCIES, 'Missing env var: CURRENCIES');

  let parsedTokens: unknown;
  try {
    parsedTokens = JSON.parse(TOKENS!);
  } catch {
    throw new Error('TOKENS is not valid JSON');
  }
  assert(
    Array.isArray(parsedTokens),
    'TOKENS must be a JSON array of objects',
  );
  type RawToken = { symbol?: unknown; chain?: unknown; address?: unknown };
  const tokenArray = parsedTokens as RawToken[];
  tokenArray.forEach((t, i) => {
    assert(
      typeof t.symbol === 'string' && t.symbol.length > 0,
      `TOKENS[${i}].symbol must be a non-empty string`,
    );
    assert(
      typeof t.chain === 'string' && t.chain.length > 0,
      `TOKENS[${i}].chain must be a non-empty string`,
    );
    assert(
      typeof t.address === 'string' && t.address.length > 0,
      `TOKENS[${i}].address must be a non-empty string`,
    );
  });

  let parsedCurrencies: unknown;
  try {
    parsedCurrencies = JSON.parse(CURRENCIES!);
  } catch {
    throw new Error('CURRENCIES is not valid JSON');
  }
  assert(
    Array.isArray(parsedCurrencies),
    'CURRENCIES must be a JSON array of strings',
  );
  const currencies = parsedCurrencies as unknown[];
  currencies.forEach((c, i) => {
    assert(
      typeof c === 'string' && c.length > 0,
      `CURRENCIES[${i}] must be a non-empty string`,
    );
  });
  assert(
    currencies.length > 0,
    'CURRENCIES must contain at least one currency',
  );

  const host = HOST && HOST.trim().length > 0 ? HOST : 'localhost';

  const portNum = PORT !== undefined
    ? Number(PORT)
    : 3000;
  assert(
    Number.isInteger(portNum),
    `PORT must be an integer, got "${PORT}"`,
  );
  // <1024 is privileged port
  assert(
    portNum >= 1024 && portNum <= 65_535,
    `PORT must be between 1 and 65535, got ${portNum}`,
  );

  const tokens = (tokenArray as Token[]).reduce<Record<string, Token[]>>(
    (acc, token) => {
      (acc[token.chain] ||= []).push(token);
      return acc;
    },
    {},
  );

  return {
    tokens,
    currencies: currencies as string[],
    isPro,
    apiKey,
    host,
    port: portNum,
  };
}