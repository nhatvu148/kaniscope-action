// Throwaway sample used to verify Kaniscope posts a real review. Contains
// deliberate bugs so the reviewer has something to flag.

// BUG: SQL injection — user input interpolated straight into the query.
export function findUser(db, name) {
  return db.query(`SELECT * FROM users WHERE name = '${name}'`);
}

// BUG: missing `await` — caches an unresolved Promise instead of the data.
const cache = new Map();
export async function getProfile(id) {
  if (cache.has(id)) return cache.get(id);
  const res = fetch(`/api/users/${id}`);
  const data = res.json();
  cache.set(id, data);
  return data;
}
