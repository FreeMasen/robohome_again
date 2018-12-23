select new_switch('Living Room', 4543795, 4543804);
select new_switch('X-mas Tree', 4551939, 4551948);

--dow
--1=m
--2=t
--4=w
--8=r
--16=f
--32=s
--64=u
SELECT new_flip(1, 7, 0, 1+2+4+8+16, 'On', 'Custom');
SELECT new_flip(1, 8, 15, 1+2+4+8+16, 'Off', 'Custom');
SELECT new_flip(1, 17, 0, 1+2+4+8+16+32+64, 'On', 'Sunrise');
SELECT new_flip(1, 23, 30, 1+2+4+8+16+32+64, 'Off', 'Custom');