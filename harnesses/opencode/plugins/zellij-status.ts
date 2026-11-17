import { spawn } from "node:child_process"
import { Plugin } from "@opencode/plugin"

const ZJ = process.env.ZJ_BIN || `${process.env.HOME}/.local/bin/zj`

function send(event: string, payload: Record<string, unknown> = {}) {
  try {
    const child = spawn(ZJ, ["hook", "opencode", event], {
      env: process.env,
      stdio: ["pipe", "ignore", "ignore"],
    })
    child.on("error", () => {})
    child.stdin?.end(JSON.stringify(payload))
  } catch {
    // Zellij and the hook listener are optional; never break the agent.
  }
}

export default Plugin.define({
  id: "local.zellij-status",
  async setup(ctx) {
    const busy = new Set<string>()
    let worked = false
    let latestSession = ""
    let doneTimers: ReturnType<typeof setTimeout>[] = []

    const clearDone = () => {
      for (const timer of doneTimers) clearTimeout(timer)
      doneTimers = []
    }

    const finish = (event: "Stop" | "StopFailure", error?: string) => {
      if (busy.size > 0 || !worked) return
      worked = false
      clearDone()
      // The TUI emits its own idle record when a turn ends. Send the hook
      // after it, and once more after a pause, so ours is the last record.
      const payload = { session_id: latestSession, error }
      doneTimers = [
        setTimeout(() => send(event, payload), 400),
        setTimeout(() => send(event, payload), 1500),
      ]
    }

    // Tool events are optional in older OpenCode builds; guard the V2 API.
    const toolHook = (ctx as { tool?: { hook?: unknown } }).tool?.hook
    if (typeof toolHook === "function") {
      const hook = toolHook as (
        name: string,
        callback: (event: Record<string, any>) => void,
      ) => Promise<unknown>
      void hook.call(toolHook, "execute.before", (event) => {
        const session = typeof event.sessionID === "string" ? event.sessionID : latestSession
        send("PreToolUse", {
          session_id: session,
          tool_name: event.tool,
          tool_input: event.input,
        })
      })
      void hook.call(toolHook, "execute.after", (event) => {
        const session = typeof event.sessionID === "string" ? event.sessionID : latestSession
        const failed = event.status === "error"
        send(failed ? "PostToolUseFailure" : "PostToolUse", {
          session_id: session,
          tool_name: event.tool,
          tool_input: event.input,
          tool_output: failed ? event.error?.message : event.result,
          error: failed ? event.error?.message : undefined,
        })
      })
    }

    const onEvent = (event: any) => {
      const sessionID = typeof event?.data?.sessionID === "string" ? event.data.sessionID : ""
      if (!sessionID) return
      latestSession = sessionID
      switch (event.type) {
        case "session.execution.started":
          clearDone()
          busy.add(sessionID)
          if (!worked) {
            worked = true
            send("UserPromptSubmit", { session_id: sessionID })
          }
          break
        case "session.execution.succeeded":
        case "session.execution.interrupted":
          busy.delete(sessionID)
          finish("Stop")
          break
        case "session.execution.failed":
          busy.delete(sessionID)
          finish("StopFailure", typeof event?.data?.error === "string" ? event.data.error : undefined)
          break
        case "session.idle":
          busy.delete(sessionID)
          finish("Stop")
          break
      }
    }

    const controller = new AbortController()
    void (async () => {
      try {
        for await (const event of ctx.event.subscribe({ signal: controller.signal })) {
          onEvent(event)
        }
      } catch {
        // A closed event stream only means status reporting stops.
      }
    })()
    return () => controller.abort()
  },
})
