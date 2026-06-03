const BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080'

async function request(method, path, body = null, token = null) {
  const headers = { 'Content-Type': 'application/json' }
  if (token) headers['Authorization'] = `Bearer ${token}`

  const res = await fetch(`${BASE_URL}${path}`, {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  })

  const data = await res.json().catch(() => null)

  if (!res.ok) {
    const err = new Error(data?.error || 'Ошибка запроса')
    err.status = res.status
    throw err
  }

  return data
}

export const api = {
  // Token
  validateToken: (token) => request('GET', `/api/token/validate/${token}`),

  // Submissions
  submitEmployee: (payload) => request('POST', '/api/submissions', payload),
  submitInspector: (payload, token) => request('POST', '/api/submissions', payload, token),

  // Inspector auth
  inspectorLogin: (tabel_number) =>
    request('POST', '/api/inspector/auth', { tabel_number }),

  // Admin auth
  adminLogin: (email, password) =>
    request('POST', '/api/admin/login', { email, password }),

  // Admin data
  getSubmissions: (params, token) => {
    const qs = new URLSearchParams(params).toString()
    return request('GET', `/api/admin/submissions?${qs}`, null, token)
  },
  updateStatus: (id, status, reject_reason, token) =>
    request('PATCH', `/api/admin/submissions/${id}/status`, { status, reject_reason }, token),

  // File upload (multipart)
  uploadFile: async (file, submissionToken = null) => {
    const form = new FormData()
    form.append('file', file)
    const headers = {}
    if (submissionToken) headers['Authorization'] = `Bearer ${submissionToken}`
    const res = await fetch(`${BASE_URL}/api/upload`, { method: 'POST', headers, body: form })
    const data = await res.json()
    if (!res.ok) throw new Error(data?.error || 'Ошибка загрузки файла')
    return data.url
  },
}
