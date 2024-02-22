-- Table with copies of user defined variables
_G_K = {}

setmetatable(_G, {
	__newindex = function (t, k, v)
		print("New variable [" .. tostring(k) .. '] = ' .. tostring(v))
		fltk_create_new_cell(k, tonumber(v) or tostring(v))
		rawset(_G_K, k, "") --> make a copy of user variable. It is used to update interface and not bringing functional stuff with it
		rawset(t, k ,v)
	end,
})
