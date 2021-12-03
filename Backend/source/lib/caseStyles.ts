import { snakeCase, camelCase, isObject, isArray, startCase } from 'lodash'

export function toSnakeCase(obj: any): any {
  if (isObject(obj) && !isArray(obj)) {
    const converted: any = {}
    Object.keys(obj).forEach((key: string) => {
      converted[snakeCase(key)] = toSnakeCase((obj as any)[key])
    })
    return converted
  } else if (isArray(obj)) {
    return obj.map((target: any) => toSnakeCase(target))
  }

  return obj
}

export function toPascalCase(obj: any): any {
  if (isObject(obj) && !isArray(obj)) {
    const converted: any  = {}
    Object.keys(obj).forEach((key) => {
      converted[startCase(camelCase(key)).replace(/ /g, '')] = toPascalCase((obj as any)[key])
    })
    return converted
  } else if (isArray(obj)) {
    return obj.map((target: any) => toPascalCase(target))
  }

  return obj
}
