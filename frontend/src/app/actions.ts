"use server";

import createClient from "openapi-fetch";
import type { paths, components } from "../config/api-types.gen.ts";

if (!process.env.API_URL) {
  throw new Error("API_URL is not set");
}

const client = createClient<paths>({ baseUrl: process.env.API_URL });

type ApiResponse<T> = { ok: true; data: T } | { ok: false; error: ApiError };
type ApiResponseVoid = { ok: true } | { ok: false; error: ApiError };

type ApiError =
  | { code: "unauthorized" }
  | { code: "not_found" }
  | { code: "unknown"; message?: string };

function toApiError(response: Response | undefined, error: unknown): ApiError {
  if (response?.status === 401) return { code: "unauthorized" };
  if (response?.status === 404) return { code: "not_found" };
  return { code: "unknown", message: error ? String(error) : undefined };
}

export type EventInput = components["schemas"]["EventInput"];
export type EventResponse = components["schemas"]["EventResponse"];

export async function getEvent(eventId: string): Promise<ApiResponse<EventResponse>> {
  const { data, error, response } = await client.GET("/event/{event_id}", {
    params: { path: { event_id: eventId } },
  });
  if (error || !data) {
    const apiError = toApiError(response, error);
    return { ok: false, error: apiError };
  }
  return { ok: true, data };
}

export async function createEvent(event: EventInput): Promise<ApiResponse<EventResponse>> {
  const { data, error, response } = await client.POST("/event", { body: event });
  if (error || !data) {
    const apiError = toApiError(response, error);
    return { ok: false, error: apiError };
  }
  return { ok: true, data };
}

export type PersonInput = components["schemas"]["PersonInput"];
export type PersonResponse = components["schemas"]["PersonResponse"];

export async function getPerson(
  eventId: string,
  personName: string,
  password?: string,
): Promise<ApiResponse<PersonResponse>> {
  const { data, error, response } = await client.GET("/event/{event_id}/people/{person_name}", {
    params: {
      path: {
        event_id: eventId,
        person_name: personName,
      },
    },
    headers: password ? { Authorization: `Bearer ${btoa(password)}` } : undefined,
  });
  if (error || !data) {
    const apiError = toApiError(response, error);
    return { ok: false, error: apiError };
  }
  return { ok: true, data };
}

export async function getPeople(eventId: string): Promise<ApiResponse<PersonResponse[]>> {
  const { data, error, response } = await client.GET("/event/{event_id}/people", {
    params: {
      path: {
        event_id: eventId,
      },
    },
  });
  if (error || !data) {
    const apiError = toApiError(response, error);
    return { ok: false, error: apiError };
  }
  return { ok: true, data };
}

// TODO: maybe revisit returning the patch response, if we encounter any weird drift behaviour
export async function updatePerson(
  eventId: string,
  personName: string,
  person: PersonInput,
  password?: string,
): Promise<ApiResponseVoid> {
  const { error, response } = await client.PATCH("/event/{event_id}/people/{person_name}", {
    params: {
      path: {
        event_id: eventId,
        person_name: personName,
      },
    },
    body: person,
    headers: password ? { Authorization: `Bearer ${btoa(password)}` } : undefined,
  });

  if (error) {
    const apiError = toApiError(response, error);
    return { ok: false, error: apiError };
  }
  return { ok: true };
}

export type StatsResponse = components["schemas"]["StatsResponse"];

export async function getStats(): Promise<ApiResponse<StatsResponse>> {
  const { data, error, response } = await client.GET("/stats", {
    next: { revalidate: 60 },
  });
  if (error || !data) {
    const apiError = toApiError(response, error);
    return { ok: false, error: apiError };
  }
  return { ok: true, data };
}
