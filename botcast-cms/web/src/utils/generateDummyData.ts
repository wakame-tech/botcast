// @specre 01KNM2BBT5T43CMWAX32H3K14G
interface JSONSchema {
  type?: string | string[]
  properties?: Record<string, JSONSchema>
  items?: JSONSchema
  enum?: unknown[]
  required?: string[]
  anyOf?: JSONSchema[]
  oneOf?: JSONSchema[]
  allOf?: JSONSchema[]
  minimum?: number
  maximum?: number
  format?: string
  description?: string
  additionalProperties?: boolean | JSONSchema
}

/**
 * Generate dummy data from JSON Schema
 */
export function generateDummyData(schema: JSONSchema): unknown {
  if (!schema || typeof schema !== 'object') {
    return null
  }

  const type = Array.isArray(schema.type) ? schema.type[0] : schema.type

  // Handle string type
  if (type === 'string') {
    if (schema.enum && schema.enum.length > 0) {
      return schema.enum[0]
    }
    return ''
  }

  // Handle number/integer type
  if (type === 'number' || type === 'integer') {
    if (schema.enum && schema.enum.length > 0) {
      return schema.enum[0]
    }
    return 0
  }

  // Handle boolean type
  if (type === 'boolean') {
    return false
  }

  // Handle array type
  if (type === 'array') {
    if (schema.items) {
      // Generate one example item
      return [generateDummyData(schema.items)]
    }
    return []
  }

  // Handle object type
  if (type === 'object') {
    const result: Record<string, unknown> = {}

    if (schema.properties) {
      // Generate all properties (both required and optional)
      for (const [key, propSchema] of Object.entries(schema.properties)) {
        result[key] = generateDummyData(propSchema)
      }
    }

    return result
  }

  // Handle null type
  if (type === 'null') {
    return null
  }

  // Handle anyOf/oneOf - use first option
  if (schema.anyOf && Array.isArray(schema.anyOf) && schema.anyOf.length > 0) {
    return generateDummyData(schema.anyOf[0])
  }
  if (schema.oneOf && Array.isArray(schema.oneOf) && schema.oneOf.length > 0) {
    return generateDummyData(schema.oneOf[0])
  }

  // Handle allOf - merge schemas (simplified)
  if (schema.allOf && Array.isArray(schema.allOf)) {
    let merged: Record<string, unknown> = {}
    for (const subSchema of schema.allOf) {
      const data = generateDummyData(subSchema)
      if (typeof data === 'object' && data !== null && !Array.isArray(data)) {
        merged = { ...merged, ...(data as Record<string, unknown>) }
      }
    }
    return merged
  }

  // Default fallback
  return null
}
