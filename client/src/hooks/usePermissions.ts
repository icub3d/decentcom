import { useMemo } from "react";

import { useServerStore } from "../stores/serverStore";

export const SEND_MESSAGES = 1 << 0;
export const READ_MESSAGES = 1 << 1;
export const MANAGE_MESSAGES = 1 << 2;
export const MANAGE_CHANNELS = 1 << 3;
export const MANAGE_ROLES = 1 << 4;
export const KICK_MEMBERS = 1 << 5;
export const BAN_MEMBERS = 1 << 6;
export const MANAGE_INVITES = 1 << 7;
export const MANAGE_SERVER = 1 << 8;
export const ATTACH_FILES = 1 << 9;
export const ADD_REACTIONS = 1 << 10;
export const MENTION_EVERYONE = 1 << 11;
export const VIEW_AUDIT_LOG = 1 << 12;
export const ADMINISTRATOR = 1 << 13;

function applyOverride(base: number, allow: number, deny: number): number {
  return (base | allow) & ~deny;
}

export function usePermissions(channelId?: string) {
  const currentUserId = useServerStore((state) => state.sessionUserId);
  const roles = useServerStore((state) => state.roles);
  const memberRoleIds = useServerStore((state) => state.memberRoleIdsByUserId);
  const channelOverrides = useServerStore((state) => state.channelOverridesByChannelId);

  return useMemo(() => {
    const roleById = new Map(roles.map((role) => [role.id, role]));
    const everyone = roleById.get("everyone");

    let bits = everyone?.permissions ?? 0;
    const assignedRoleIds = new Set<string>(["everyone"]);

    if (currentUserId) {
      for (const roleId of memberRoleIds[currentUserId] ?? []) {
        assignedRoleIds.add(roleId);
      }
    }

    for (const roleId of assignedRoleIds) {
      const role = roleById.get(roleId);
      if (!role) {
        continue;
      }
      bits |= role.permissions;
    }

    if ((bits & ADMINISTRATOR) === ADMINISTRATOR) {
      return {
        bits,
        has: () => true,
      };
    }

    if (channelId) {
      for (const overrideRow of channelOverrides[channelId] ?? []) {
        if (!assignedRoleIds.has(overrideRow.role_id)) {
          continue;
        }
        bits = applyOverride(bits, overrideRow.allow, overrideRow.deny);
      }
    }

    return {
      bits,
      has: (permission: number) => (bits & permission) === permission,
    };
  }, [channelId, channelOverrides, currentUserId, memberRoleIds, roles]);
}
