-- Keep Yazi's full Nerd Font icon set, but recolor all icons into the
-- obsidian & muted slate-blue palette instead of default rainbow hex colors.
function Entity:icon()
	local icon = th.icon:match(self._file, { hovered = self._file.is_hovered })
	if not icon then
		return ""
	elseif self._file.is_hovered then
		return icon.text .. " "
	end

	local color
	if self._file.cha.is_dummy then
		color = "#e06c75"
	elseif self._file.cha.is_hidden then
		color = "#61768f"
	elseif self._file.cha.is_dir then
		color = "#758a9b"
	elseif self._file.link_to then
		color = "#949ea3"
	else
		color = "#61768f"
	end

	return ui.Line(icon.text .. " "):fg(color)
end
